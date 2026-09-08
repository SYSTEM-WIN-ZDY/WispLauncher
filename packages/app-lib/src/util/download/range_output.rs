//! Direct positional output for native HTTP/1.1 range downloads.

use crate::util::io::{self, IOError};
use std::path::Path;
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::sync::Mutex;

/// A preallocated `.part` file shared by non-overlapping range tasks.
pub(crate) struct RangeOutput {
    file: Mutex<File>,
}

impl RangeOutput {
    pub(crate) async fn create(
        path: &Path,
        size: u64,
    ) -> Result<Arc<Self>, IOError> {
        let file = io::retry_windows_sharing_violation(
            path,
            "creating ranged download output",
            || async {
                let file = OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .read(true)
                    .write(true)
                    .open(path)
                    .await?;
                file.set_len(size).await?;
                Ok(file)
            },
        )
        .await
        .map_err(|error| io::io_error_with_lock_info(error, path))?;
        Ok(Arc::new(Self {
            file: Mutex::new(file),
        }))
    }

    /// Writes a validated range chunk at its absolute file offset.
    pub(crate) async fn write_at(
        &self,
        offset: u64,
        bytes: &[u8],
        path: &Path,
    ) -> Result<(), IOError> {
        let mut file = self.file.lock().await;
        file.seek(SeekFrom::Start(offset))
            .await
            .map_err(|error| io::io_error_with_lock_info(error, path))?;
        file.write_all(bytes)
            .await
            .map_err(|error| io::io_error_with_lock_info(error, path))
    }

    pub(crate) async fn flush(&self, path: &Path) -> Result<(), IOError> {
        self.file
            .lock()
            .await
            .flush()
            .await
            .map_err(|error| io::io_error_with_lock_info(error, path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn writes_non_overlapping_ranges_into_a_preallocated_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("ranged.part");
        let output = RangeOutput::create(&path, 12).await.unwrap();

        output.write_at(8, b"ijkl", &path).await.unwrap();
        output.write_at(0, b"abcd", &path).await.unwrap();
        output.write_at(4, b"efgh", &path).await.unwrap();
        output.flush(&path).await.unwrap();

        assert_eq!(tokio::fs::read(path).await.unwrap(), b"abcdefghijkl");
    }
}
