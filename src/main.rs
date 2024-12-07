use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use tokio::task;
use walkdir::WalkDir;

#[tokio::main]
async fn main() {
    let log_dir = "/Users/carlwang/temp/logs/bar-svc-offline-app";
    let content_keyword = "AssetMarketInfoService";
    let filename_keyword = Some("error");

    let result = find_files_with_keyword(log_dir,
                                         content_keyword,
                                         filename_keyword).await;
    let result = result.unwrap();
    if result.is_empty() {
        println!("没有找到合适的文件")
    } else {
        for file in result {
            println!("{:?}", file)
        }
    }
}

async fn find_files_with_keyword(log_dir: &str, content_keyword: &str, filename_keyword: Option<&str>) ->
io::Result<Vec<PathBuf>> {

    let mut tasks = Vec::new();

    for entry in WalkDir::new(log_dir) {
        let entry = entry?;
        let path = entry.path().to_path_buf();
        if !path.is_file() {
            continue;
        }

        if let Some(keyword) = filename_keyword {
            if !path.file_name().unwrap_or_default().to_string_lossy().contains(keyword) {
                continue;
            }
        }
        // 创建异步任务处理文件
        let content_keyword = content_keyword.to_string();
        tasks.push(task::spawn_blocking(move || {
            if contains_keyword(&path, &content_keyword).unwrap_or(false) {
                Some(path)
            } else {
                None
            }
        }));
    }

    let mut matching_files = Vec::new();
    for task in tasks {
        if let Some(path) = task.await? {
            matching_files.push(path);
        }
    }

    Ok(matching_files)
}

fn contains_keyword(file_path: &Path, keyword: &str) -> io::Result<bool> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.contains(keyword) {
            return Ok(true);
        }
    }
    Ok(false)
}
