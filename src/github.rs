use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::Value;

pub struct Github {
    client: Client,
}

impl Github {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Validate the revision to be a SHA-1 hash (40 characters of hex)
    pub fn sha(rev: &str) -> bool {
        if rev.len() != 40 || !rev.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }

        true
    }

    /// Format the source to be a valid GitHub URL
    pub fn format_source(source: &str) -> String {
        // Replace a SSH URL with a HTTPS URL
        let source = source.replace("git@github.com:", "https://github.com/");
        // Remove the .git suffix if it exists
        let source = source.trim_end_matches(".git");

        source.to_string()
    }

    /// Extract owner and repo from the GitHub URL
    pub fn split_source(source: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = source
            .trim_start_matches("https://github.com/")
            .split('/')
            .collect();

        if parts.len() != 2 {
            return Err(anyhow!("Invalid GitHub URL format"));
        }

        // Return the owner and repo
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Check if a repository exists and return the default branch
    pub async fn check_repo(&self, owner: &str, repo: &str) -> Option<String> {
        let url = format!("https://api.github.com/repos/{owner}/{repo}");

        match self
            .client
            .get(&url)
            .header("User-Agent", "Rust Binary")
            .send()
            .await
        {
            Ok(response) => {
                let branch = match response.json::<Value>().await {
                    Ok(json) => json["default_branch"].as_str().map(|s| s.to_string()),
                    Err(_) => None,
                };

                branch
            }
            Err(_) => None,
        }
    }

    /// Check if a file exists at a given path and branch
    pub async fn file_exists(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        ref_: &str,
    ) -> Result<bool> {
        let url = format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}?ref={ref_}");

        // TODO: can this return a success for a file that does not exist
        let response = self
            .client
            .head(&url)
            .header("User-Agent", "Rust Binary")
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Format a GitHub URL for a file at a given revision
    pub fn web_url(owner: &str, repo: &str, file_path: &str, revision: &str) -> String {
        format!("https://github.com/{owner}/{repo}/blob/{revision}/{file_path}")
    }
}
