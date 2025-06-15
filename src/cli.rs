use crate::parser::{DEFAULT_REVISION, DEFAULT_SOURCE, OLD_REVISION, OLD_SOURCE, Parser};
use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde_json::Value;
use std::process::Command;

#[derive(clap::Parser)]
pub struct Cli {
    /// Docker image name with optional tag (e.g., nginx:latest)
    pub image: String,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        // TODO: if this finds a ref it will open a page which may 404
        if let Ok((msg, url)) = inspect_image(&self.image) {
            println!("{msg}");
            open_url(&url)?;

            return Ok(());
        }

        // TODO: tag is unused for now
        let (name, _tag) = Parser::split_tag(&self.image)?;
        let parts: Vec<&str> = name.split('/').collect();

        // TODO: assumes max 3 parts
        let (registry, namespace, repo) = match parts.as_slice() {
            [repo] => (None, "library", *repo),
            [namespace, repo] => (None, *namespace, *repo),
            [registry, namespace, repo] => (Some(*registry), *namespace, *repo),
            _ => unimplemented!("Unsupported image name. Assumes a name with max 2 slashes atm"),
        };

        let client = Client::new();

        match (registry, namespace, repo) {
            (None, "library", repo) => {
                let exists = check_dockerhub(&client, "library", repo).await;
                if exists {
                    let url = format!("https://hub.docker.com/_/{repo}");
                    println!("Opening Docker Hub → {url}");
                    open_url(&url)?;
                } else {
                    bail!("Docker Hub repo does not exist");
                }
            }
            (None, namespace, repo) => {
                let url = format!("https://api.github.com/repos/{namespace}/{repo}");
                let request = client.get(&url).header("User-Agent", "Rust Binary");

                let response = request.send().await;

                if let Ok(response) = response {
                    if response.status().is_success() {
                        let branch = match response.json::<Value>().await {
                            Ok(json) => json["default_branch"].as_str().map(|s| s.to_string()),
                            Err(_) => None,
                        };

                        let paths = vec!["Dockerfile", "docker/Dockerfile"];
                        let mut file_path = None;
                        let mut repo_branch = None;

                        for path in paths {
                            let (dockerfile_exists, branch) = if let Some(ref branch) = branch {
                                let branch = match branch.as_str() {
                                    "main" => "main".to_string(),
                                    "master" => "master".to_string(),
                                    _ => branch.clone(),
                                };

                                (
                                    check_github(&client, namespace, repo, path, &branch)
                                        .await
                                        .unwrap_or(false),
                                    branch,
                                )
                            } else {
                                let exists = check_github(&client, namespace, repo, path, "main")
                                    .await
                                    .unwrap_or(false);

                                if exists {
                                    (true, "main".to_string())
                                } else {
                                    (
                                        check_github(&client, namespace, repo, path, "master")
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

                        let url = if let Some(file_path) = file_path {
                            format!(
                                "https://github.com/{namespace}/{repo}/blob/{}/{file_path}",
                                repo_branch.unwrap()
                            )
                        } else {
                            format!("https://github.com/{namespace}/{repo}")
                        };

                        println!("Opening GitHub → {url}");
                        open_url(&url)?;
                    } else {
                        let exists = check_dockerhub(&client, namespace, repo).await;
                        if exists {
                            let url = format!("https://hub.docker.com/r/{namespace}/{repo}");
                            println!("Opening Docker Hub → {url}");
                            open_url(&url)?;
                        } else {
                            bail!("Docker Hub repo does not exist");
                        }
                    }
                } else {
                    let exists = check_dockerhub(&client, namespace, repo).await;
                    if exists {
                        let url = format!("https://hub.docker.com/r/{namespace}/{repo}");
                        println!("Opening Docker Hub → {url}");
                        open_url(&url)?;
                    } else {
                        bail!("Docker Hub repo does not exist");
                    }
                }
            }
            (Some(registry), namespace, repo) => {
                // TODO: if this is ghcr.io then maybe we should open the repo page instead, or try to find the dockerfile in the root and open that
                let url = format!("https://{registry}/{namespace}/{repo}");
                println!("Opening registry → {url}");
                open_url(&url)?;
            }
        }

        Ok(())
    }
}

fn inspect_image(image: &str) -> Result<(String, String)> {
    // Check if you have a local image and inspect its labels to construct a url
    let source = Parser::label(image, DEFAULT_SOURCE).or_else(|| Parser::label(image, OLD_SOURCE));

    // TODO: revision may not be a commit hash, check if it's a sha commit hash, if it is then use it otherwise ignore
    let revision =
        Parser::label(image, DEFAULT_REVISION).or_else(|| Parser::label(image, OLD_REVISION));

    if let (Some(src), Some(rev)) = (source, revision) {
        // Replace a SSH URL with a HTTPS URL
        let src = src.replace("git@github.com:", "https://github.com/");
        // Remove the .git suffix if it exists
        let src = src.trim_end_matches(".git");

        // TODO: add a check to see if the file exists at the url
        // Assume there is a dockerfile at the root of the repo
        // TODO: even with src and rev the repo may not have a Dockerfile here
        let url = format!("{src}/blob/{rev}/Dockerfile");
        let msg = format!("Opening GitHub → {url}");
        return Ok((msg, url));
    }

    Err(anyhow::anyhow!(""))
}

async fn check_github(
    client: &Client,
    owner: &str,
    repo: &str,
    path: &str,
    branch: &str,
) -> Result<bool> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}?ref={branch}");

    // TODO: maybe don't error out here and let caller continue
    let response = client
        .get(&url)
        .header("User-Agent", "Rust Binary")
        .send()
        .await?;

    Ok(response.status().is_success())
}

async fn check_dockerhub(client: &Client, namespace: &str, repo: &str) -> bool {
    let url = format!(
        "https://hub.docker.com/v2/repositories/{}/{}/",
        namespace, repo
    );
    match client.get(&url).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

fn open_url(url: &str) -> Result<()> {
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
