use reqwest::Client;

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

    pub async fn check(&self, namespace: &str, repo: &str) -> bool {
        let url = format!(
            "https://hub.docker.com/v2/repositories/{}/{}/",
            namespace, repo
        );
        match self.client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    pub fn url(&self, namespace: Option<&str>, repo: &str) -> String {
        if let Some(namespace) = namespace {
            format!("https://hub.docker.com/r/{namespace}/{repo}")
        } else {
            format!("https://hub.docker.com/_/{repo}")
        }
    }
}
