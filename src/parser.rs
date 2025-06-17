use anyhow::{Result, bail};
use std::process::Command;

pub struct Parser;

impl Parser {
    pub fn parse(image: &str) -> Result<(Option<&str>, Option<&str>, &str)> {
        // TODO: tag is unused for now
        let (name, _tag) = Self::split_tag(image)?;
        let parts: Vec<&str> = name.split('/').collect();

        // TODO: assumes max 3 parts
        let (registry, namespace, repo) = match parts.as_slice() {
            [repo] => (None, None, *repo),
            [namespace, repo] => (None, Some(*namespace), *repo),
            [registry, namespace, repo] => (Some(*registry), Some(*namespace), *repo),
            _ => unimplemented!("Unsupported image name. Assumes a name with max 2 slashes atm"),
        };

        Ok((registry, namespace, repo))
    }

    pub fn label(image: &str, label: &str) -> Option<String> {
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

    fn split_tag(image: &str) -> Result<(&str, Option<&str>)> {
        // TODO: ignore private repos for now and assume it only contains a tag at the end
        if let Some(slash) = image.find('/') {
            let host = &image[..slash];
            // TODO: probably doesn't cover all cases
            if host.contains(':') || host.contains("localhost") || host.contains("127.0.0.1") {
                bail!("private registries not supported: {host}");
            }
        }

        if let Some(idx) = image.rfind(':') {
            if image[idx..].find('/').is_none() {
                return Ok((&image[..idx], Some(&image[idx + 1..])));
            }
        }
        Ok((image, None))
    }
}
