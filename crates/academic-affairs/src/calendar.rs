use anyhow::{Context, Result, anyhow};
use scraper::{Html, Selector};
use serde::Serialize;

const CALENDAR_URL: &str = "https://jw.nju.edu.cn/qxnjxxl/list.htm";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Calendar {
    pub title: String,
    pub page_url: String,
    pub pdf_urls: Vec<String>,
    pub image_urls: Vec<String>,
}

/// 获取教务网当前「全学年教学校历」页面中的 PDF 和图片链接。
pub async fn get_calendar(client: &common::Client) -> Result<Calendar> {
    let html = client
        .get(CALENDAR_URL)
        .send()
        .await
        .context("failed to request academic calendar page")?
        .error_for_status()
        .context("academic calendar page returned an error status")?
        .text()
        .await
        .context("failed to read academic calendar page")?;

    parse_calendar(&html, CALENDAR_URL)
}

fn parse_calendar(html: &str, page_url: &str) -> Result<Calendar> {
    let document = Html::parse_document(html);
    let base_url =
        reqwest::Url::parse(page_url).with_context(|| format!("invalid page URL: {page_url}"))?;
    let title_selector = selector(".arti_title")?;
    let pdf_link_selector = selector("a[href$='.pdf'], a[href*='.pdf?']")?;
    let pdf_player_selector = selector(".wp_pdf_player")?;
    let image_selector = selector(".read img, .wp_articlecontent img, img[data-layer='photo']")?;

    let title = document
        .select(&title_selector)
        .next()
        .map(|title| text_content(title.text()))
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| "南京大学校历".to_string());
    let mut pdf_urls = Vec::new();
    let mut image_urls = Vec::new();

    for element in document.select(&pdf_link_selector) {
        if let Some(href) = element.value().attr("href") {
            push_unique_url(&mut pdf_urls, &base_url, href);
        }
    }

    for element in document.select(&pdf_player_selector) {
        if let Some(pdf_src) = element.value().attr("pdfsrc") {
            push_unique_url(&mut pdf_urls, &base_url, pdf_src);
        }

        if let Some(src) = element.value().attr("src") {
            if let Ok(viewer_url) = base_url.join(src) {
                if let Some((_, file)) = viewer_url.query_pairs().find(|(name, _)| name == "file") {
                    push_unique_url(&mut pdf_urls, &base_url, file.as_ref());
                }
            }
        }
    }

    for image in document.select(&image_selector) {
        let Some(src) = image
            .value()
            .attr("original-src")
            .or_else(|| image.value().attr("data-imgsrc"))
            .or_else(|| image.value().attr("src"))
        else {
            continue;
        };

        if src.contains("/icon_pdf.") || src.contains("/_visitcount") {
            continue;
        }

        push_unique_url(&mut image_urls, &base_url, src);
    }

    if pdf_urls.is_empty() {
        return Err(anyhow!("no PDF URL found in academic calendar page"));
    }
    if image_urls.is_empty() {
        return Err(anyhow!("no image URL found in academic calendar page"));
    }

    Ok(Calendar {
        title,
        page_url: base_url.to_string(),
        pdf_urls,
        image_urls,
    })
}

fn push_unique_url(urls: &mut Vec<String>, base_url: &reqwest::Url, url: &str) {
    if let Ok(url) = base_url.join(url) {
        let url = url.to_string();
        if !urls.contains(&url) {
            urls.push(url);
        }
    }
}

fn text_content<'a>(text: impl Iterator<Item = &'a str>) -> String {
    text.collect::<String>().trim().to_string()
}

fn selector(selector: &str) -> Result<Selector> {
    Selector::parse(selector).map_err(|error| anyhow!("invalid CSS selector {selector}: {error}"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_calendar_links() {
        let html = r#"
            <h1 class="arti_title">南京大学2026-2027学年校历</h1>
            <div class="read">
                <p><img src="/_ueditor/themes/default/images/icon_pdf.gif" /><a href="/_upload/article/files/a/calendar.pdf">南京大学2026-2027校历.pdf</a></p>
                <p><img data-layer="photo" src="/_upload/article/images/a/calendar.png" original-src="/_upload/article/images/a/calendar_d.png" /></p>
                <div class="wp_pdf_player" pdfsrc="/_upload/article/files/a/player.pdf"></div>
            </div>
        "#;

        let calendar = parse_calendar(html, "https://jw.nju.edu.cn/qxnjxxl/list.htm").unwrap();

        assert_eq!(calendar.title, "南京大学2026-2027学年校历");
        assert_eq!(
            calendar.pdf_urls,
            vec![
                "https://jw.nju.edu.cn/_upload/article/files/a/calendar.pdf",
                "https://jw.nju.edu.cn/_upload/article/files/a/player.pdf",
            ]
        );
        assert_eq!(
            calendar.image_urls,
            vec!["https://jw.nju.edu.cn/_upload/article/images/a/calendar_d.png"]
        );
    }

    #[tokio::test]
    async fn test_get_calendar() {
        let client = common::ClientBuilder::new(reqwest::Client::new()).build();
        let calendar = get_calendar(&client).await.unwrap();

        assert!(!calendar.pdf_urls.is_empty());
        assert!(!calendar.image_urls.is_empty());
    }
}
