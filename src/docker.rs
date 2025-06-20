use reqwest::Client;
use std::process::Command;

pub const DEFAULT_SOURCE: &str = "org.opencontainers.image.source";
pub const DEFAULT_REVISION: &str = "org.opencontainers.image.revision";
pub const OLD_SOURCE: &str = "org.label-schema.vcs-url";
pub const OLD_REVISION: &str = "org.label-schema.vcs-ref";

pub struct Docker {
    client: Client,
}

impl Docker {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn inspect(image: &str, label: &str) -> Option<String> {
        let output = Command::new("docker")
            .args([
                "inspect",
                "--format",
                &format!("{{{{index .Config.Labels \"{}\"}}}}", label),
                image,
            ])
            .output()
            .ok()?;

        if output.status.success() {
            let label_value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !label_value.is_empty() {
                return Some(label_value);
            }
        }

        None
    }

    pub async fn repo_exists(&self, namespace: &str, repo: &str) -> bool {
        let url = Self::api_url(namespace, repo);

        match self.client.head(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    pub fn web_url(namespace: Option<&str>, repo: &str) -> String {
        if let Some(namespace) = namespace {
            Self::web_namespace_url(namespace, repo)
        } else {
            Self::web_repo_url(repo)
        }
    }

    fn api_url(namespace: &str, repo: &str) -> String {
        format!("https://hub.docker.com/v2/repositories/{namespace}/{repo}")
    }

    fn web_namespace_url(namespace: &str, repo: &str) -> String {
        format!("https://hub.docker.com/r/{namespace}/{repo}")
    }

    fn web_repo_url(repo: &str) -> String {
        format!("https://hub.docker.com/_/{repo}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPO: &str = "project";
    const NAMESPACE: &str = "namespace";

    mod public {
        use super::*;

        // TODO: unsure how to test Docker::inspect() while using an image that doesn't exist. Need to mock somehow without changing code above
        // TODO: similarly, Docker::repo_exists() needs to be mocked and resolve to some mock server

        #[test]
        fn test_web_url_namespace() {
            assert_eq!(
                Docker::web_url(Some("namespace"), REPO),
                "https://hub.docker.com/r/namespace/project"
            );
        }

        #[test]
        fn test_web_url_no_namespace() {
            assert_eq!(
                Docker::web_url(None, REPO),
                "https://hub.docker.com/_/project"
            );
        }
    }

    mod private {
        use super::*;

        #[test]
        fn test_api_url() {
            assert_eq!(
                Docker::api_url(NAMESPACE, REPO),
                "https://hub.docker.com/v2/repositories/namespace/project"
            );
        }

        #[test]
        fn test_web_namespace_url() {
            assert_eq!(
                Docker::web_namespace_url(NAMESPACE, REPO),
                "https://hub.docker.com/r/namespace/project"
            );
        }

        #[test]
        fn test_web_repo_url() {
            assert_eq!(
                Docker::web_repo_url(REPO),
                "https://hub.docker.com/_/project"
            );
        }
    }
}
