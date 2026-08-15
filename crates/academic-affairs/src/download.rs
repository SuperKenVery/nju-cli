use anyhow::{Context, Result, anyhow};
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;

use crate::SITE_BASE_URL;

const DOWNLOAD_EXTENSIONS: &[&str] = &[
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "zip", "rar", "7z", "jpg", "jpeg", "png",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadFile {
    pub title: String,
    pub url: String,
}

/// 获取教务网下载专区文章对应的实际可下载文件。
///
/// `url` 可以是列表接口返回的文章页地址，也可以是直接指向附件的相对/绝对 URL。
pub async fn list_article_download_files(
    client: &common::Client,
    url: &str,
) -> Result<Vec<DownloadFile>> {
    let url = reqwest::Url::parse(SITE_BASE_URL)
        .context("invalid academic affairs site base URL")?
        .join(url)
        .with_context(|| format!("invalid academic affairs download URL: {url}"))?;

    if is_downloadable_url(&url) {
        let url = normalize_download_url(url);
        return Ok(vec![DownloadFile {
            title: file_name_from_url(&url).unwrap_or_else(|| "download".to_string()),
            url: url.to_string(),
        }]);
    }

    let response = client
        .get(url.clone())
        .send()
        .await
        .with_context(|| format!("failed to request academic affairs download page: {url}"))?
        .error_for_status()
        .with_context(|| {
            format!("academic affairs download page returned an error status: {url}")
        })?;
    let page_url = response.url().clone();
    let html = response
        .text()
        .await
        .with_context(|| format!("failed to read academic affairs download page: {url}"))?;

    parse_download_files(&html, page_url.as_str())
}

fn parse_download_files(html: &str, page_url: &str) -> Result<Vec<DownloadFile>> {
    let document = Html::parse_document(html);
    let base_url = reqwest::Url::parse(page_url)
        .with_context(|| format!("invalid download page URL: {page_url}"))?;
    let content_selector = selector(".wp_articlecontent, .read, .col_news_con")?;
    let link_selector = selector("a[href]")?;
    let pdf_player_selector = selector(".wp_pdf_player, [pdfsrc]")?;
    let image_selector = selector("img")?;
    let mut files = Vec::new();

    let content_elements: Vec<ElementRef<'_>> = document.select(&content_selector).collect();
    let roots = if content_elements.is_empty() {
        document.root_element().select(&selector("body")?).collect()
    } else {
        content_elements
    };

    for root in roots {
        for element in root.select(&link_selector) {
            let Some(href) = element.value().attr("href") else {
                continue;
            };
            let Some(url) = base_url.join(href).ok().filter(is_downloadable_url) else {
                continue;
            };
            let title = download_title(element);
            push_unique_file(&mut files, url, title);
        }

        for element in root.select(&pdf_player_selector) {
            if let Some(pdf_src) = element.value().attr("pdfsrc") {
                if let Ok(url) = base_url.join(pdf_src) {
                    push_unique_file(&mut files, url, download_title(element));
                }
            }

            if let Some(src) = element.value().attr("src") {
                if let Ok(viewer_url) = base_url.join(src) {
                    if let Some((_, file)) =
                        viewer_url.query_pairs().find(|(name, _)| name == "file")
                    {
                        if let Ok(url) = base_url.join(file.as_ref()) {
                            push_unique_file(&mut files, url, download_title(element));
                        }
                    }
                }
            }
        }

        for image in root.select(&image_selector) {
            let Some(src) = image
                .value()
                .attr("original-src")
                .or_else(|| image.value().attr("data-imgsrc"))
                .or_else(|| image.value().attr("src"))
            else {
                continue;
            };
            if !is_article_image(src) {
                continue;
            }
            if let Ok(url) = base_url.join(src) {
                push_unique_file(&mut files, url, download_title(image));
            }
        }
    }

    Ok(files)
}

fn download_title(element: ElementRef<'_>) -> Option<String> {
    element
        .value()
        .attr("sudyfile-attr")
        .and_then(sudy_file_title)
        .or_else(|| element.value().attr("download").map(str::to_string))
        .or_else(|| element.value().attr("title").map(str::to_string))
        .or_else(|| {
            let text = text_content(element.text());
            (!text.is_empty()).then_some(text)
        })
        .map(|title| title.trim().to_string())
        .filter(|title| !title.is_empty())
}

fn sudy_file_title(attr: &str) -> Option<String> {
    for marker in ["'title':'", "\"title\":\""] {
        let Some(start) = attr.find(marker) else {
            continue;
        };
        let rest = &attr[start + marker.len()..];
        let quote = marker.chars().last()?;
        let end = rest.find(quote)?;
        let title = rest[..end].trim();
        if !title.is_empty() {
            return Some(title.to_string());
        }
    }

    None
}

fn push_unique_file(files: &mut Vec<DownloadFile>, url: reqwest::Url, title: Option<String>) {
    if !is_downloadable_url(&url) {
        return;
    }

    let url = normalize_download_url(url).to_string();
    if files.iter().any(|file| file.url == url) {
        return;
    }

    let title = title.unwrap_or_else(|| {
        reqwest::Url::parse(&url)
            .ok()
            .and_then(|url| file_name_from_url(&url))
            .unwrap_or_else(|| "download".to_string())
    });

    files.push(DownloadFile { title, url });
}

fn normalize_download_url(mut url: reqwest::Url) -> reqwest::Url {
    // 部分旧附件链接写成 webplus.nju.edu.cn，但同一路径可从教务网域名访问；
    // 统一到 jw.nju.edu.cn，避免 webplus 域名解析失败。
    if url.host_str() == Some("webplus.nju.edu.cn") && url.path().starts_with("/_upload/") {
        let _ = url.set_host(Some("jw.nju.edu.cn"));
    }

    url
}

fn is_article_image(src: &str) -> bool {
    let src = src.to_ascii_lowercase();
    src.contains("/_upload/article/images/")
        && !src.contains("/icon_")
        && (src.ends_with(".jpg") || src.ends_with(".jpeg") || src.ends_with(".png"))
}

fn is_downloadable_url(url: &reqwest::Url) -> bool {
    let path = url.path().to_ascii_lowercase();
    DOWNLOAD_EXTENSIONS
        .iter()
        .any(|extension| path.ends_with(&format!(".{extension}")))
}

fn file_name_from_url(url: &reqwest::Url) -> Option<String> {
    url.path_segments()?
        .next_back()
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
}

fn text_content<'a>(text: impl Iterator<Item = &'a str>) -> String {
    text.collect::<String>().trim().to_string()
}

fn selector(selector: &str) -> Result<Selector> {
    Selector::parse(selector).map_err(|error| anyhow!("invalid CSS selector {selector}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_download_files_from_article_page() {
        let html = r#"
            <div class="read"><div class="wp_articlecontent">
                <p><a href="/_upload/article/files/a/form.doc" sudyfile-attr="{'title':'申请表.doc'}">申请表.doc</a></p>
                <p><img data-layer="photo" src="/_upload/article/images/a/calendar.png" original-src="/_upload/article/images/a/calendar_d.png" sudyfile-attr="{'title':'校历.png'}" /></p>
                <div class="wp_pdf_player" pdfsrc="/_upload/article/files/a/preview.pdf"></div>
                <p><img src="/_ueditor/themes/default/images/icon_doc.gif" /></p>
            </div></div>
        "#;

        let files = parse_download_files(html, "https://jw.nju.edu.cn/a/b/page.htm").unwrap();

        assert_eq!(
            files,
            vec![
                DownloadFile {
                    title: "申请表.doc".to_string(),
                    url: "https://jw.nju.edu.cn/_upload/article/files/a/form.doc".to_string(),
                },
                DownloadFile {
                    title: "preview.pdf".to_string(),
                    url: "https://jw.nju.edu.cn/_upload/article/files/a/preview.pdf".to_string(),
                },
                DownloadFile {
                    title: "校历.png".to_string(),
                    url: "https://jw.nju.edu.cn/_upload/article/images/a/calendar_d.png"
                        .to_string(),
                },
            ]
        );
    }
}
