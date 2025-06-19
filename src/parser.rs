pub struct Parser;

impl Parser {
    pub fn parse_image(image: &str) -> (Option<&str>, Vec<&str>, Option<&str>) {
        let (name, reference) = Self::split_ref(image);
        let mut parts: Vec<&str> = name.split('/').collect();

        let registry = if parts.len() > 1
            && (parts[0].contains('.') || parts[0].contains(':') || parts[0].contains("localhost"))
        {
            let registry = parts.remove(0);
            if registry == "docker.io" {
                None
            } else {
                Some(registry)
            }
        } else {
            None
        };

        if registry.is_none() && parts.len() == 1 {
            parts.insert(0, "library");
        }

        (registry, parts, reference)
    }

    fn split_ref(image: &str) -> (&str, Option<&str>) {
        if let Some(idx) = image.rfind('@') {
            return (&image[..idx], Some(&image[idx + 1..]));
        }
        if let Some(idx) = image.rfind(':') {
            if image[idx..].find('/').is_none() {
                return (&image[..idx], Some(&image[idx + 1..]));
            }
        }
        (image, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_repo() -> Result<()> {
        assert_eq!(
            Parser::parse_image("project"),
            (None, vec!["library", "project"], None)
        );
        Ok(())
    }

    #[test]
    fn test_repo_with_tag() -> Result<()> {
        assert_eq!(
            Parser::parse_image("project:latest"),
            (None, vec!["library", "project"], Some("latest"))
        );
        Ok(())
    }

    #[test]
    fn test_repo_with_hash() -> Result<()> {
        assert_eq!(
            Parser::parse_image("project@sha256:1234"),
            (None, vec!["library", "project"], Some("sha256:1234"))
        );
        Ok(())
    }

    #[test]
    fn test_repo_with_namespace() -> Result<()> {
        assert_eq!(
            Parser::parse_image("namespace/project"),
            (None, vec!["namespace", "project"], None)
        );
        Ok(())
    }

    #[test]
    fn test_repo_with_namespace_with_tag() -> Result<()> {
        assert_eq!(
            Parser::parse_image("namespace/project:latest"),
            (None, vec!["namespace", "project"], Some("latest"))
        );
        Ok(())
    }

    #[test]
    fn test_repo_with_namespace_with_hash() -> Result<()> {
        assert_eq!(
            Parser::parse_image("namespace/project@sha256:1234"),
            (None, vec!["namespace", "project"], Some("sha256:1234"))
        );
        Ok(())
    }

    #[test]
    fn test_registry_with_namespace() -> Result<()> {
        assert_eq!(
            Parser::parse_image("registry.io/namespace"),
            (Some("registry.io"), vec!["namespace"], None)
        );
        Ok(())
    }

    #[test]
    fn test_registry_with_namespace_with_repo() -> Result<()> {
        assert_eq!(
            Parser::parse_image("registry.io/namespace/project"),
            (Some("registry.io"), vec!["namespace", "project"], None)
        );
        Ok(())
    }

    #[test]
    fn test_registry_with_namespace_with_repo_with_tag() -> Result<()> {
        assert_eq!(
            Parser::parse_image("registry.io/namespace/project:latest"),
            (
                Some("registry.io"),
                vec!["namespace", "project"],
                Some("latest")
            )
        );
        Ok(())
    }

    #[test]
    fn test_registry_with_namespace_with_repo_with_hash() -> Result<()> {
        assert_eq!(
            Parser::parse_image("registry.io/namespace/project@sha256:1234"),
            (
                Some("registry.io"),
                vec!["namespace", "project"],
                Some("sha256:1234")
            )
        );
        Ok(())
    }
}
