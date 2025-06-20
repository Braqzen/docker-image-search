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
    ///
    /// SAFETY: assumes source does not have an extension (e.g. .git)
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
    ///
    /// SAFETY: "default_branch" is in response
    pub async fn check_repo(&self, owner: &str, repo: &str) -> Option<String> {
        let url = Self::repo_url(owner, repo);

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
        let url = Self::file_url(owner, repo, path, ref_);

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

    fn repo_url(owner: &str, repo: &str) -> String {
        format!("https://api.github.com/repos/{owner}/{repo}")
    }

    fn file_url(owner: &str, repo: &str, path: &str, ref_: &str) -> String {
        format!("https://api.github.com/repos/{owner}/{repo}/{path}?ref={ref_}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: &str = "owner";
    const REPO: &str = "repo";
    const PATH: &str = "path";
    const REF: &str = "ref";

    mod public {
        use super::*;

        // TODO: unsure about check_repo() and file_exists() like in docker.rs

        #[test]
        fn test_sha() {
            assert!(Github::sha("1234567890123456789012345678901234567890"));
        }

        #[test]
        fn test_sha_invalid_length() {
            assert!(!Github::sha("1234567890"));
        }

        #[test]
        fn test_sha_invalid_characters() {
            assert!(!Github::sha("123456789012345678901234567890123456789*"));
        }

        #[test]
        fn test_format_source() {
            assert_eq!(
                Github::format_source("git@github.com:owner/repo.git"),
                "https://github.com/owner/repo"
            );
        }

        #[test]
        fn test_split_source() -> Result<()> {
            assert_eq!(
                Github::split_source(&format!("https://github.com/{OWNER}/{REPO}"))?,
                (OWNER.to_string(), REPO.to_string())
            );

            Ok(())
        }

        #[test]
        fn test_split_source_invalid() {
            let error = Github::split_source("https://github.com/owner");
            assert!(error.is_err());
        }

        #[test]
        fn test_web_url() {
            assert_eq!(
                Github::web_url(OWNER, REPO, PATH, REF),
                "https://github.com/owner/repo/blob/ref/path"
            );
        }
    }

    mod private {
        use super::*;

        #[test]
        fn test_repo_url() {
            assert_eq!(
                Github::repo_url(OWNER, REPO),
                "https://api.github.com/repos/owner/repo"
            );
        }

        #[test]
        fn test_file_url() {
            assert_eq!(
                Github::file_url(OWNER, REPO, PATH, REF),
                "https://api.github.com/repos/owner/repo/path?ref=ref"
            );
        }
    }
}
