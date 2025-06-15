use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

pub struct Github {
    client: Client,
}

impl Github {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn check_repo(&self, owner: &str, repo: &str) -> (bool, Option<String>) {
        let url = format!("https://api.github.com/repos/{owner}/{repo}");

        match self
            .client
            .get(&url)
            .header("User-Agent", "Rust Binary")
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status().is_success();
                let branch = match response.json::<Value>().await {
                    Ok(json) => json["default_branch"].as_str().map(|s| s.to_string()),
                    Err(_) => None,
                };

                (status, branch)
            }
            Err(_) => (false, None),
        }
    }

    pub async fn check(&self, owner: &str, repo: &str, path: &str, branch: &str) -> Result<bool> {
        let url =
            format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}?ref={branch}");

        // TODO: maybe don't error out here and let caller continue
        let response = self
            .client
            .get(&url)
            .header("User-Agent", "Rust Binary")
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    pub fn url(
        namespace: &str,
        repo: &str,
        file_path: Option<&str>,
        repo_branch: Option<&str>,
    ) -> String {
        if let Some(file_path) = file_path {
            format!(
                "https://github.com/{namespace}/{repo}/blob/{}/{file_path}",
                repo_branch.unwrap()
            )
        } else {
            format!("https://github.com/{namespace}/{repo}")
        }
    }
}
