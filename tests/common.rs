//! Common test utilities and helpers

use std::process::Command;

/// Get an access token from gcloud for authenticating to Google Cloud APIs.
///
/// This function runs `gcloud auth print-access-token` to get the current
/// user's access token. This requires that:
/// 1. gcloud CLI is installed
/// 2. User is authenticated (`gcloud auth login`)
/// 3. Application default credentials are set up
///
/// # Returns
///
/// Returns the access token as a String, or an error if unable to get the token.
///
/// # Example
///
/// ```rust
/// let token = common::get_gcloud_access_token().expect("Failed to get gcloud token");
/// // Use token with Bearer authentication
/// ```
pub fn get_gcloud_access_token() -> anyhow::Result<String> {
    let output = Command::new("gcloud")
        .args(&["auth", "print-access-token"])
        .output()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to run gcloud command. Make sure gcloud CLI is installed and in PATH: {}",
                e
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "gcloud auth print-access-token failed: {}. Make sure you're logged in with 'gcloud auth login'",
            stderr
        ));
    }

    let token = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("Failed to parse gcloud output: {}", e))?
        .trim()
        .to_string();

    if token.is_empty() {
        return Err(anyhow::anyhow!(
            "gcloud returned empty token. Run 'gcloud auth login' to authenticate"
        ));
    }

    Ok(token)
}

/// Get the default Google Cloud project ID from gcloud config.
///
/// # Returns
///
/// Returns the project ID as a String, or an error if unable to get it.
pub fn get_gcloud_project() -> anyhow::Result<String> {
    let output = Command::new("gcloud")
        .args(&["config", "get-value", "project"])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run gcloud command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "gcloud config get-value project failed: {}",
            stderr
        ));
    }

    let project = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("Failed to parse gcloud output: {}", e))?
        .trim()
        .to_string();

    if project.is_empty() || project == "(unset)" {
        return Err(anyhow::anyhow!(
            "No default project set. Run 'gcloud config set project PROJECT_ID'"
        ));
    }

    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Only run when gcloud is available
    fn test_gcloud_access_token() {
        let result = get_gcloud_access_token();
        if let Ok(token) = result {
            assert!(!token.is_empty());
            // Access tokens are typically JWT format or similar
            assert!(token.len() > 10);
        }
    }

    #[test]
    #[ignore] // Only run when gcloud is available
    fn test_gcloud_project() {
        let result = get_gcloud_project();
        if let Ok(project) = result {
            assert!(!project.is_empty());
        }
    }
}

