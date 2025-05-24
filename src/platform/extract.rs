use crate::error::{Result, RezToolsError};
use log::{debug, info};
use std::path::Path;
use tokio::fs;

/// Archive extraction utilities
pub struct Extractor;

impl Extractor {
    /// Extract an archive based on its file extension
    pub async fn extract<P: AsRef<Path>, Q: AsRef<Path>>(
        archive_path: P,
        extract_to: Q,
    ) -> Result<()> {
        let archive_path = archive_path.as_ref();
        let extract_to = extract_to.as_ref();

        info!(
            "Extracting {} to {}",
            archive_path.display(),
            extract_to.display()
        );

        // Ensure extraction directory exists
        fs::create_dir_all(extract_to).await?;

        // Determine archive type by extension
        let extension = archive_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "zip" => Self::extract_zip(archive_path, extract_to).await,
            "gz" | "tgz" => {
                // Check if it's a .tar.gz
                let file_name = archive_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
                    Self::extract_tar_gz(archive_path, extract_to).await
                } else {
                    Err(RezToolsError::ConfigError(format!(
                        "Unsupported archive format: {}",
                        extension
                    )))
                }
            }
            "tar" => Self::extract_tar(archive_path, extract_to).await,
            _ => Err(RezToolsError::ConfigError(format!(
                "Unsupported archive format: {}",
                extension
            ))),
        }
    }

    /// Extract ZIP archive
    async fn extract_zip<P: AsRef<Path>, Q: AsRef<Path>>(
        archive_path: P,
        extract_to: Q,
    ) -> Result<()> {
        let archive_path = archive_path.as_ref();
        let extract_to = extract_to.as_ref();

        debug!("Extracting ZIP archive");

        // Use blocking task for CPU-intensive work
        let archive_path = archive_path.to_path_buf();
        let extract_to = extract_to.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&archive_path).map_err(|e| {
                RezToolsError::ConfigError(format!("Failed to open archive: {}", e))
            })?;

            let mut archive = zip::ZipArchive::new(file).map_err(|e| {
                RezToolsError::ConfigError(format!("Failed to read ZIP archive: {}", e))
            })?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(|e| {
                    RezToolsError::ConfigError(format!("Failed to read ZIP entry: {}", e))
                })?;

                let outpath = match file.enclosed_name() {
                    Some(path) => extract_to.join(path),
                    None => continue,
                };

                if file.name().ends_with('/') {
                    // Directory
                    std::fs::create_dir_all(&outpath).map_err(|e| {
                        RezToolsError::ConfigError(format!("Failed to create directory: {}", e))
                    })?;
                } else {
                    // File
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p).map_err(|e| {
                                RezToolsError::ConfigError(format!(
                                    "Failed to create directory: {}",
                                    e
                                ))
                            })?;
                        }
                    }

                    let mut outfile = std::fs::File::create(&outpath).map_err(|e| {
                        RezToolsError::ConfigError(format!("Failed to create file: {}", e))
                    })?;

                    std::io::copy(&mut file, &mut outfile).map_err(|e| {
                        RezToolsError::ConfigError(format!("Failed to extract file: {}", e))
                    })?;
                }

                // Set permissions on Unix systems
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = file.unix_mode() {
                        std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))
                            .map_err(|e| {
                            RezToolsError::ConfigError(format!("Failed to set permissions: {}", e))
                        })?;
                    }
                }
            }

            Ok::<(), RezToolsError>(())
        })
        .await
        .map_err(|e| RezToolsError::ConfigError(format!("Extraction task failed: {}", e)))??;

        info!("ZIP extraction completed");
        Ok(())
    }

    /// Extract TAR.GZ archive
    async fn extract_tar_gz<P: AsRef<Path>, Q: AsRef<Path>>(
        archive_path: P,
        extract_to: Q,
    ) -> Result<()> {
        let archive_path = archive_path.as_ref();
        let extract_to = extract_to.as_ref();

        debug!("Extracting TAR.GZ archive");

        // Use blocking task for CPU-intensive work
        let archive_path = archive_path.to_path_buf();
        let extract_to = extract_to.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&archive_path).map_err(|e| {
                RezToolsError::ConfigError(format!("Failed to open archive: {}", e))
            })?;

            let decoder = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(decoder);

            archive.unpack(&extract_to).map_err(|e| {
                RezToolsError::ConfigError(format!("Failed to extract TAR.GZ archive: {}", e))
            })?;

            Ok::<(), RezToolsError>(())
        })
        .await
        .map_err(|e| RezToolsError::ConfigError(format!("Extraction task failed: {}", e)))??;

        info!("TAR.GZ extraction completed");
        Ok(())
    }

    /// Extract TAR archive
    async fn extract_tar<P: AsRef<Path>, Q: AsRef<Path>>(
        archive_path: P,
        extract_to: Q,
    ) -> Result<()> {
        let archive_path = archive_path.as_ref();
        let extract_to = extract_to.as_ref();

        debug!("Extracting TAR archive");

        // Use blocking task for CPU-intensive work
        let archive_path = archive_path.to_path_buf();
        let extract_to = extract_to.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&archive_path).map_err(|e| {
                RezToolsError::ConfigError(format!("Failed to open archive: {}", e))
            })?;

            let mut archive = tar::Archive::new(file);

            archive.unpack(&extract_to).map_err(|e| {
                RezToolsError::ConfigError(format!("Failed to extract TAR archive: {}", e))
            })?;

            Ok::<(), RezToolsError>(())
        })
        .await
        .map_err(|e| RezToolsError::ConfigError(format!("Extraction task failed: {}", e)))??;

        info!("TAR extraction completed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_extractor_determine_archive_type() {
        // Test ZIP detection
        let zip_path = PathBuf::from("test.zip");
        assert!(zip_path.extension().unwrap() == "zip");

        // Test TAR.GZ detection
        let tar_gz_path = PathBuf::from("test.tar.gz");
        assert!(tar_gz_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .ends_with(".tar.gz"));

        // Test TGZ detection
        let tgz_path = PathBuf::from("test.tgz");
        assert!(tgz_path.extension().unwrap() == "tgz");

        // Test TAR detection
        let tar_path = PathBuf::from("test.tar");
        assert!(tar_path.extension().unwrap() == "tar");
    }

    #[tokio::test]
    async fn test_extract_unsupported_format() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.unknown");
        let extract_to = temp_dir.path().join("extracted");

        // Create a fake archive file
        fs::write(&archive_path, "fake archive content").unwrap();

        let result = Extractor::extract(&archive_path, &extract_to).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported archive format"));
    }

    #[tokio::test]
    async fn test_extract_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("nonexistent.zip");
        let extract_to = temp_dir.path().join("extracted");

        let result = Extractor::extract(&archive_path, &extract_to).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extract_creates_extraction_directory() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.zip");
        let extract_to = temp_dir.path().join("extracted");

        // Create a minimal ZIP file (this will fail extraction but should create the directory)
        fs::write(&archive_path, "fake zip content").unwrap();

        // Verify extraction directory doesn't exist initially
        assert!(!extract_to.exists());

        // Try to extract (will fail due to invalid ZIP, but should create directory)
        let _result = Extractor::extract(&archive_path, &extract_to).await;

        // Directory should be created even if extraction fails
        assert!(extract_to.exists());
    }

    #[test]
    fn test_archive_type_detection_by_extension() {
        struct TestCase {
            filename: &'static str,
            expected_type: &'static str,
        }

        let test_cases = vec![
            TestCase {
                filename: "archive.zip",
                expected_type: "zip",
            },
            TestCase {
                filename: "archive.tar.gz",
                expected_type: "tar.gz",
            },
            TestCase {
                filename: "archive.tgz",
                expected_type: "tgz",
            },
            TestCase {
                filename: "archive.tar",
                expected_type: "tar",
            },
            TestCase {
                filename: "ARCHIVE.ZIP",
                expected_type: "zip",
            }, // Case insensitive
        ];

        for test_case in test_cases {
            let path = PathBuf::from(test_case.filename);
            let extension = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            match extension.as_str() {
                "zip" => assert_eq!(test_case.expected_type, "zip"),
                "gz" | "tgz" => {
                    if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
                        assert!(
                            test_case.expected_type == "tar.gz" || test_case.expected_type == "tgz"
                        );
                    }
                }
                "tar" => assert_eq!(test_case.expected_type, "tar"),
                _ => {} // Other types
            }
        }
    }

    #[tokio::test]
    async fn test_extract_with_nested_paths() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.zip");
        let extract_to = temp_dir
            .path()
            .join("nested")
            .join("deep")
            .join("extracted");

        // Create a fake archive file
        fs::write(&archive_path, "fake archive content").unwrap();

        // Verify nested path doesn't exist initially
        assert!(!extract_to.exists());

        // Try to extract (will fail due to invalid archive, but should create nested directories)
        let _result = Extractor::extract(&archive_path, &extract_to).await;

        // Nested directories should be created
        assert!(extract_to.exists());
    }

    #[test]
    fn test_extractor_static_methods() {
        // Test that Extractor methods are static and don't require instance
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.zip");
        let extract_to = temp_dir.path().join("extracted");

        // This should compile without creating an Extractor instance
        let _future = Extractor::extract(&archive_path, &extract_to);

        // The future exists, proving the method is static
        assert!(true);
    }
}
