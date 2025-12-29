use tokio::time::{interval, Duration};
use futures_util::future::join_all;

pub async fn start_cleanup_task(uploads_dir: String) {
    let mut interval = interval(Duration::from_secs(3600)); // Toutes les heures

    loop {
        interval.tick().await;
        cleanup_old_files(&uploads_dir).await;
    }
}

async fn cleanup_old_files(uploads_dir: &str) {
    let now = std::time::SystemTime::now();
    let cutoff = Duration::from_secs(7 * 24 * 3600); // 7 jours

    if let Ok(mut entries) = tokio::fs::read_dir(uploads_dir).await {
        let mut tasks = vec![];

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();

            if let Ok(metadata) = entry.metadata().await {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(age) = now.duration_since(modified) {
                        if age > cutoff {
                            tasks.push(tokio::fs::remove_file(path));
                        }
                    }
                }
            }
        }

        // Suppression parallèle
        let _ = join_all(tasks).await;
    }
}
