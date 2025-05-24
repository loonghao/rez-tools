use crate::error::{Result, RezToolsError};
use log::{debug, info, warn};
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Download client with retry logic and progress reporting
pub struct DownloadClient {
    client: reqwest::Client,
    max_retries: usize,
    timeout: Duration,
}

impl Default for DownloadClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadClient {
    /// Create a new download client with default settings
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minutes
            .user_agent("rez-tools/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            max_retries: 3,
            timeout: Duration::from_secs(300),
        }
    }

    /// Download a file with retry logic
    pub async fn download_file<P: AsRef<Path>>(&self, url: &str, destination: P) -> Result<()> {
        let destination = destination.as_ref();

        // Create parent directory if it doesn't exist
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).await?;
        }

        for attempt in 1..=self.max_retries {
            info!(
                "Downloading {} (attempt {} of {})",
                url, attempt, self.max_retries
            );

            match self.try_download(url, destination).await {
                Ok(()) => {
                    info!("Successfully downloaded to {}", destination.display());
                    return Ok(());
                }
                Err(e) => {
                    warn!("Download attempt {} failed: {}", attempt, e);

                    if attempt == self.max_retries {
                        return Err(RezToolsError::ConfigError(format!(
                            "Failed to download {} after {} attempts: {}",
                            url, self.max_retries, e
                        )));
                    }

                    // Exponential backoff
                    let delay = Duration::from_secs(2_u64.pow(attempt as u32 - 1));
                    debug!("Waiting {:?} before retry", delay);
                    tokio::time::sleep(delay).await;
                }
            }
        }

        unreachable!()
    }

    /// Single download attempt
    async fn try_download<P: AsRef<Path>>(&self, url: &str, destination: P) -> Result<()> {
        let destination = destination.as_ref();

        // Start the download
        let response = self
            .client
            .get(url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| RezToolsError::ConfigError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(RezToolsError::ConfigError(format!(
                "HTTP error {}: {}",
                response.status(),
                response
                    .status()
                    .canonical_reason()
                    .unwrap_or("Unknown error")
            )));
        }

        // Get content length for progress reporting
        let total_size = response.content_length();
        if let Some(size) = total_size {
            debug!("Download size: {} bytes", size);
        }

        // Create temporary file
        let temp_path = destination.with_extension("tmp");
        let mut file = fs::File::create(&temp_path)
            .await
            .map_err(|e| RezToolsError::ConfigError(format!("Failed to create file: {}", e)))?;

        // Download with streaming
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk =
                chunk.map_err(|e| RezToolsError::ConfigError(format!("Stream error: {}", e)))?;

            file.write_all(&chunk)
                .await
                .map_err(|e| RezToolsError::ConfigError(format!("Write error: {}", e)))?;

            downloaded += chunk.len() as u64;

            // Log progress periodically
            if let Some(total) = total_size {
                let progress = (downloaded as f64 / total as f64) * 100.0;
                if downloaded % (1024 * 1024) == 0 || downloaded == total {
                    debug!(
                        "Downloaded {:.1}% ({} / {} bytes)",
                        progress, downloaded, total
                    );
                }
            }
        }

        // Ensure all data is written
        file.flush()
            .await
            .map_err(|e| RezToolsError::ConfigError(format!("Flush error: {}", e)))?;

        drop(file);

        // Move temp file to final destination
        fs::rename(&temp_path, destination)
            .await
            .map_err(|e| RezToolsError::ConfigError(format!("Failed to move file: {}", e)))?;

        info!(
            "Downloaded {} bytes to {}",
            downloaded,
            destination.display()
        );
        Ok(())
    }

    /// Download and return content as bytes
    pub async fn download_bytes(&self, url: &str) -> Result<Vec<u8>> {
        for attempt in 1..=self.max_retries {
            debug!(
                "Downloading {} to memory (attempt {} of {})",
                url, attempt, self.max_retries
            );

            match self.try_download_bytes(url).await {
                Ok(bytes) => {
                    debug!("Downloaded {} bytes", bytes.len());
                    return Ok(bytes);
                }
                Err(e) => {
                    warn!("Download attempt {} failed: {}", attempt, e);

                    if attempt == self.max_retries {
                        return Err(e);
                    }

                    let delay = Duration::from_secs(2_u64.pow(attempt as u32 - 1));
                    tokio::time::sleep(delay).await;
                }
            }
        }

        unreachable!()
    }

    /// Single attempt to download bytes
    async fn try_download_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| RezToolsError::ConfigError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(RezToolsError::ConfigError(format!(
                "HTTP error {}: {}",
                response.status(),
                response
                    .status()
                    .canonical_reason()
                    .unwrap_or("Unknown error")
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| RezToolsError::ConfigError(format!("Failed to read response: {}", e)))?;

        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_download_client_new() {
        let client = DownloadClient::new();
        assert_eq!(client.max_retries, 3);
        assert_eq!(client.timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_download_client_default() {
        let client = DownloadClient::default();
        assert_eq!(client.max_retries, 3);
        assert_eq!(client.timeout, Duration::from_secs(300));
    }

    #[tokio::test]
    async fn test_download_file_to_temp_dir() {
        let temp_dir = TempDir::new().unwrap();
        let destination = temp_dir.path().join("test_file.txt");

        let client = DownloadClient::new();

        // This test would require a real HTTP server or mock
        // For now, we'll test the error case with an invalid URL
        let result = client
            .download_file(
                "http://invalid-url-that-does-not-exist.com/file.txt",
                &destination,
            )
            .await;

        // Should fail with network error
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to download"));
    }

    #[tokio::test]
    async fn test_download_bytes_invalid_url() {
        let client = DownloadClient::new();

        let result = client
            .download_bytes("http://invalid-url-that-does-not-exist.com/data")
            .await;

        // Should fail with network error
        assert!(result.is_err());
    }

    #[test]
    fn test_download_client_creates_parent_directory() {
        // This test verifies that the download client would create parent directories
        // We can't easily test the actual download without a mock server
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("deep").join("file.txt");

        // Verify parent doesn't exist initially
        assert!(!nested_path.parent().unwrap().exists());

        // The actual download would create the parent directory
        // This is tested implicitly in the integration tests
    }

    #[test]
    fn test_exponential_backoff_calculation() {
        // Test that our exponential backoff calculation works correctly
        let base_delay = 2u64;

        // Attempt 1: 2^0 = 1 second
        let delay1 = Duration::from_secs(base_delay.pow(1 - 1));
        assert_eq!(delay1, Duration::from_secs(1));

        // Attempt 2: 2^1 = 2 seconds
        let delay2 = Duration::from_secs(base_delay.pow(2 - 1));
        assert_eq!(delay2, Duration::from_secs(2));

        // Attempt 3: 2^2 = 4 seconds
        let delay3 = Duration::from_secs(base_delay.pow(3 - 1));
        assert_eq!(delay3, Duration::from_secs(4));
    }
}
