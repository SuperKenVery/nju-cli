use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use percent_encoding::percent_decode_str;
use reqwest::header::CONTENT_DISPOSITION;

#[derive(Debug, Args)]
pub struct DownloadCommand {
    /// 要下载的文件 URL，可以是原始地址，也可以是使用 --vpn 时页面里出现的 Web VPN 地址。
    pub url: String,
    /// 输出文件或目录；默认保存到当前目录，文件名取自响应头或 URL 路径。
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

pub async fn handle(command: DownloadCommand, client: &common::Client) -> Result<()> {
    download(client, &command.url, command.output.as_deref()).await
}

async fn download(client: &common::Client, url: &str, output: Option<&Path>) -> Result<()> {
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to request {url}"))?
        .error_for_status()
        .with_context(|| format!("download returned an error status: {url}"))?;

    let file_name = response_file_name(response.headers()).or_else(|| file_name_from_url(url));
    let path = output_path(output, file_name.as_deref())?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }

    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("failed to read response body: {url}"))?;
    std::fs::write(&path, bytes).with_context(|| format!("failed to write {}", path.display()))?;

    println!("{}", path.display());
    Ok(())
}

fn output_path(output: Option<&Path>, file_name: Option<&str>) -> Result<PathBuf> {
    let file_name = file_name.unwrap_or("download");

    match output {
        Some(path) if path.is_dir() => Ok(path.join(file_name)),
        Some(path) => Ok(path.to_path_buf()),
        None => Ok(PathBuf::from(file_name)),
    }
}

fn response_file_name(headers: &reqwest::header::HeaderMap) -> Option<String> {
    let value = headers.get(CONTENT_DISPOSITION)?.to_str().ok()?;
    let mut fallback: Option<String> = None;

    for part in value.split(';').skip(1) {
        let part = part.trim();
        let Some((name, value)) = part.split_once('=') else {
            continue;
        };
        let name = name.trim();
        let value = value.trim();

        if name.eq_ignore_ascii_case("filename*") {
            if let Some((_, encoded)) = value.rsplit_once("''") {
                if let Ok(decoded) = percent_decode_str(encoded).decode_utf8() {
                    let decoded = decoded.trim();
                    if !decoded.is_empty() {
                        return Some(sanitize_filename::sanitize(decoded));
                    }
                }
            }
        } else if name.eq_ignore_ascii_case("filename") && fallback.is_none() {
            fallback = parse_filename(value);
        }
    }

    fallback
        .filter(|name| !name.is_empty())
        .map(|name| sanitize_filename::sanitize(&name))
}

fn parse_filename(value: &str) -> Option<String> {
    let value = value.trim();

    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        let inner = &value[1..value.len() - 1];
        Some(inner.replace("\\\"", "\""))
    } else if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn file_name_from_url(url: &str) -> Option<String> {
    let url = reqwest::Url::parse(url).ok()?;
    let segment = url.path_segments()?.next_back()?;

    if segment.is_empty() {
        return None;
    }

    let decoded = percent_decode_str(segment).decode_utf8().ok()?;
    let decoded = decoded.trim();
    if decoded.is_empty() {
        return None;
    }

    Some(sanitize_filename::sanitize(decoded))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_web_vpn_file_name_from_url() {
        let name = file_name_from_url(
            "https://jw-nju-edu-cn.atrust.nju.edu.cn/_upload/article/files/a/%E6%A0%A1%E5%8E%86.pdf",
        )
        .unwrap();

        assert_eq!(name, "校历.pdf");
    }

    #[test]
    fn falls_back_to_url_path_when_url_has_no_path() {
        let name = file_name_from_url("https://example.com");
        assert_eq!(name, None);
    }
}
