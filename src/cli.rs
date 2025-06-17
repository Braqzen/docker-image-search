use crate::{
    docker::{DEFAULT_REVISION, DEFAULT_SOURCE, Docker, OLD_REVISION, OLD_SOURCE},
    github::Github,
    parser::Parser,
};
use anyhow::{Context, Result, bail};
use std::process::Command;

#[derive(clap::Parser)]
pub struct Cli {
    /// Docker image name with optional tag (e.g., nginx:latest)
    pub image: String,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        // TODO: if this finds a ref it will open a page which may 404
        if let Ok((msg, url)) = inspect_image(&self.image).await {
            println!("aaa {msg}");
            open(&url)?;

            return Ok(());
        }

        let docker = Docker::new();
        let github = Github::new();

        match Parser::parse(&self.image)? {
            (None, None, repo) => {
                if !docker.check("library", repo).await {
                    bail!("Docker Hub repo does not exist");
                }

                let url = docker.url(None, repo);
                println!("Opening Docker Hub → {url}");
                open(&url)?;
            }
            (None, Some(namespace), repo) => {
                let (exists, branch) = github.check_repo(namespace, repo).await;
                if exists {
                    let paths = vec!["Dockerfile", "docker/Dockerfile"];
                    let mut file_path = None;
                    let mut repo_branch = None;

                    for path in paths {
                        let (dockerfile_exists, branch) = if let Some(ref branch) = branch {
                            (
                                github
                                    .check(namespace, repo, path, &branch)
                                    .await
                                    .unwrap_or(false),
                                branch.clone(),
                            )
                        } else {
                            if github
                                .check(namespace, repo, path, "main")
                                .await
                                .unwrap_or(false)
                            {
                                (true, "main".to_string())
                            } else {
                                (
                                    github
                                        .check(namespace, repo, path, "master")
                                        .await
                                        .unwrap_or(false),
                                    "master".to_string(),
                                )
                            }
                        };

                        if dockerfile_exists {
                            file_path = Some(path.to_string());
                            repo_branch = Some(branch);
                            break;
                        };
                    }

                    let url = Github::url(
                        namespace,
                        repo,
                        file_path.as_deref(),
                        repo_branch.as_deref(),
                    );

                    println!("Opening GitHub → {url}");
                    open(&url)?;
                } else {
                    if !docker.check(namespace, repo).await {
                        bail!("Docker Hub repo does not exist");
                    }

                    let url = docker.url(Some(namespace), repo);
                    println!("Opening Docker Hub → {url}");
                    open(&url)?;
                }
            }
            (Some(registry), Some(namespace), repo) => {
                // TODO: if this is ghcr.io then maybe we should open the repo page instead, or try to find the dockerfile in the root and open that
                let url = format!("https://{registry}/{namespace}/{repo}");
                println!("Opening registry → {url}");
                open(&url)?;
            }
            _ => unreachable!(),
        }

        Ok(())
    }
}

async fn inspect_image(image: &str) -> Result<(String, String)> {
    // Check if you have a local image and inspect its labels to construct a url
    let source = Parser::label(image, DEFAULT_SOURCE).or_else(|| Parser::label(image, OLD_SOURCE));
    let revision =
        Parser::label(image, DEFAULT_REVISION).or_else(|| Parser::label(image, OLD_REVISION));

    if let (Some(src), Some(rev)) = (source, revision) {
        // Validate that the revision is a valid SHA-1 hash (40 characters of hex)
        if rev.len() != 40 || !rev.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(anyhow::anyhow!("Invalid SHA-1 revision format"));
        }

        // Replace a SSH URL with a HTTPS URL
        let src = src.replace("git@github.com:", "https://github.com/");
        // Remove the .git suffix if it exists
        let src = src.trim_end_matches(".git");

        // Extract owner and repo from the GitHub URL
        let parts: Vec<&str> = src
            .trim_start_matches("https://github.com/")
            .split('/')
            .collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid GitHub URL format"));
        }
        let (owner, repo) = (parts[0], parts[1]);

        // Check if the Dockerfile exists at this commit
        let github = Github::new();
        if !github.check(owner, repo, "Dockerfile", &rev).await? {
            return Err(anyhow::anyhow!("Dockerfile not found at this commit"));
        }

        let url = format!("{src}/blob/{rev}/Dockerfile");
        let msg = format!("Opening GitHub → {url}");
        return Ok((msg, url));
    }

    Err(anyhow::anyhow!(""))
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
