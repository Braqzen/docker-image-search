use crate::{
    docker::{DEFAULT_REVISION, DEFAULT_SOURCE, Docker, OLD_REVISION, OLD_SOURCE},
    github::Github,
    parser::Parser,
};
use anyhow::{Context, Result, bail};
use std::process::Command;

#[derive(clap::Parser)]
pub struct Cli {
    /// Docker image name with optional tag (e.g., project:reference)
    pub image: String,

    /// GitHub username
    #[clap(env = "GITHUB_USER", hide_env = true)]
    pub user: String,

    /// GitHub token with read access to packages
    #[clap(env = "GITHUB_TOKEN", hide_env = true)]
    pub token: String,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        // Check if there is a local image and inspect its labels to construct a url
        let source = Docker::inspect(&self.image, DEFAULT_SOURCE)
            .or_else(|| Docker::inspect(&self.image, OLD_SOURCE));
        let revision = Docker::inspect(&self.image, DEFAULT_REVISION)
            .or_else(|| Docker::inspect(&self.image, OLD_REVISION));

        if let (Some(source), Some(revision)) = (source, revision) {
            if Github::sha(&revision) {
                let source = Github::format_source(&source);
                if let Ok((owner, repo)) = Github::split_source(&source) {
                    if let Ok(true) = Github::new()
                        .file_exists(&owner, &repo, "Dockerfile", &revision)
                        .await
                    {
                        let url = Github::web_url(&owner, &repo, "Dockerfile", &revision);
                        println!("Opening {url}");
                        return open(&url);
                    }
                }
            }
        }

        let (registry, parts, reference) = Parser::parse_image(&self.image);

        let url = self.url(registry, parts, reference).await?;
        println!("Opening {url}");
        open(&url)?;

        Ok(())
    }

    async fn url(
        &self,
        registry: Option<&str>,
        parts: Vec<&str>,
        reference: Option<&str>,
    ) -> Result<String> {
        let docker = Docker::new();
        let github = Github::new();

        match (registry, parts.as_slice()) {
            // Case 1: Docker Hub library image (e.g., "project:reference")
            (None, ["library", repo]) => {
                if !docker.repo_exists("library", repo).await {
                    bail!("Docker Hub repo does not exist");
                }

                Ok(Docker::web_url(None, repo))
            }

            // Case 2: No registry, namespace/repo - GitHub || Docker (e.g., "project/repo:reference")
            (None, [namespace, repo]) => {
                if let Some(default_branch) = github.check_repo(namespace, repo).await {
                    // Couple paths to check for Dockerfile
                    let paths = vec!["Dockerfile", "docker/Dockerfile"];

                    // Found path to Dockerfile
                    let mut file_path = None;

                    // If reference is provided, use it to get the revision
                    let revision = if let Some(reference) = reference {
                        github
                            .revision(
                                namespace,
                                repo,
                                reference,
                                &self.user,
                                &self.token,
                                &default_branch,
                            )
                            .await
                            .unwrap_or(default_branch)
                    } else {
                        default_branch
                    };

                    for path in paths {
                        if github
                            .file_exists(namespace, repo, path, &revision)
                            .await
                            .unwrap_or(false)
                        {
                            file_path = Some(path.to_string());
                            break;
                        }
                    }

                    if let Some(file_path) = file_path {
                        return Ok(Github::web_url(namespace, repo, &file_path, &revision));
                    }
                }

                if !docker.repo_exists(namespace, repo).await {
                    bail!("Docker Hub repo does not exist");
                }

                Ok(Docker::web_url(Some(namespace), repo))
            }

            // Case 3: Registry with namespace (e.g., "registry.io/project")
            (Some(_registry), [_namespace]) => {
                bail!("Registry with only a namespace is an invalid image format");
            }

            // Case 4: Registry with namespace and repo (e.g., "registry.io/project/repo:reference")
            (Some(registry), [namespace, repo]) => {
                // TODO: if this is ghcr.io then maybe we should open the repo page instead, or try to find the dockerfile in the root and open that
                Ok(format!("https://{registry}/{namespace}/{repo}"))
            }

            _ => unimplemented!("Unsupported image format"),
        }
    }
}

fn open(url: &str) -> Result<()> {
    let output = Command::new("setsid")
        .arg("xdg-open")
        .arg(url)
        .output()
        .context("Failed to execute xdg-open")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to open URL: {}", error);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const USER: &str = "user";
    const TOKEN: &str = "token";

    const REGISTRY: &str = "registry.io";
    const NAMESPACE: &str = "project";
    const REPO: &str = "repo";
    const REFERENCE: &str = "reference";

    // TODO: case 1 & 2 make http calls that are not mocked so skipping tests for now

    #[tokio::test]
    #[should_panic]
    async fn test_registry_with_namespace() {
        let cli = Cli {
            image: format!("{REGISTRY}/{NAMESPACE}").to_string(),
            user: USER.to_string(),
            token: TOKEN.to_string(),
        };

        cli.url(Some(REGISTRY), vec![NAMESPACE], None)
            .await
            .expect("Unsupported image format");
    }

    #[tokio::test]
    async fn test_registry_with_namespace_and_repo() -> Result<()> {
        let cli = Cli {
            image: format!("{REGISTRY}/{NAMESPACE}/{REPO}").to_string(),
            user: USER.to_string(),
            token: TOKEN.to_string(),
        };

        let url = cli.url(Some(REGISTRY), vec![NAMESPACE, REPO], None).await?;
        assert_eq!(url, format!("https://{REGISTRY}/{NAMESPACE}/{REPO}"));

        Ok(())
    }

    #[tokio::test]
    async fn test_registry_with_namespace_and_repo_and_reference() -> Result<()> {
        // TODO: reference is unused
        let cli = Cli {
            image: format!("{REGISTRY}/{NAMESPACE}/{REPO}:{REFERENCE}").to_string(),
            user: USER.to_string(),
            token: TOKEN.to_string(),
        };

        let url = cli.url(Some(REGISTRY), vec![NAMESPACE, REPO], None).await?;
        assert_eq!(url, format!("https://{REGISTRY}/{NAMESPACE}/{REPO}"));

        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn test_unsupported_image_format() {
        let cli = Cli {
            image: format!("{NAMESPACE}/{REPO}/subdir:{REFERENCE}").to_string(),
            user: USER.to_string(),
            token: TOKEN.to_string(),
        };

        cli.url(None, vec![NAMESPACE, REPO, "subdir"], None)
            .await
            .expect("Unsupported image format");
    }
}
