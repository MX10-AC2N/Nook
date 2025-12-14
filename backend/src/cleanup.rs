use tokio::time::{interval, Duration};
use std::path::Path;

pub async fn start_cleanup_task(uploads_dir: &str) {
    let mut interval = interval(Duration::from_secs(3600)); // Toutes les heures
    loop {
        interval.tick().await;
        cleanup_old_files(uploads_dir).await;
    }
}

async fn cleanup_old_files(uploads_dir: &str) {
    let now = std::time::SystemTime::now();
    if let Ok(entries) = tokio::fs::read_dir(uploads_dir).await {
        let mut tasks = vec![];
        let cutoff = Duration::from_secs(7 * 24 * 3600); // 7 jours

        tokio::pin!(entries);
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if let Ok(metadata) = tokio::fs::metadata(&path).await {
                if let Ok(modified) = metadata.modified() {
                    if now.duration_since(modified).unwrap_or(cutoff) > cutoff {
                        tasks.push(tokio::fs::remove_file(path));
                    }
                }
            }
        }
        futures_util::future::join_all(tasks).await;
    }
}
