use common::{ClientBuilder, html_to_markdown, html_to_markdown_with_base_url, read_html_page};

#[test]
fn converts_html_to_markdown() {
    let markdown = html_to_markdown("<h1>标题</h1><p>正文</p>").unwrap();

    assert!(markdown.contains("标题"));
    assert!(markdown.contains("正文"));
}

#[test]
fn absolutizes_relative_image_urls() {
    let markdown = html_to_markdown_with_base_url(
        r#"<p><img src="images/a.png" alt="图片"></p><p><img src="/_upload/b.png" alt="上传"></p>"#,
        "https://jw.nju.edu.cn/ggtz/page.html",
    )
    .unwrap();

    assert!(markdown.contains("![图片](https://jw.nju.edu.cn/ggtz/images/a.png)"));
    assert!(markdown.contains("![上传](https://jw.nju.edu.cn/_upload/b.png)"));
}

#[test]
fn keeps_absolute_image_urls() {
    let markdown = html_to_markdown_with_base_url(
        r#"<img src="https://example.com/a.png" alt="图片">"#,
        "https://jw.nju.edu.cn/ggtz/page.html",
    )
    .unwrap();

    assert!(markdown.contains("![图片](https://example.com/a.png)"));
}

#[test]
fn absolutizes_relative_link_urls() {
    let markdown = html_to_markdown_with_base_url(
        r#"<a href="files/a.pdf">附件</a><a href="/_upload/b.pdf">上传附件</a>"#,
        "https://jw.nju.edu.cn/ggtz/page.html",
    )
    .unwrap();

    assert!(markdown.contains("[附件](https://jw.nju.edu.cn/ggtz/files/a.pdf)"));
    assert!(markdown.contains("[上传附件](https://jw.nju.edu.cn/_upload/b.pdf)"));
}

#[test]
fn keeps_image_title_when_absolutizing_urls() {
    let markdown = html_to_markdown_with_base_url(
        r#"<img src="a.png" alt="图片" title="标题">"#,
        "https://jw.nju.edu.cn/ggtz/page.html",
    )
    .unwrap();

    assert!(
        markdown.contains("![图片](https://jw.nju.edu.cn/ggtz/a.png \"标题\")"),
        "{markdown}"
    );
}

#[test]
fn keeps_angle_wrapped_image_urls() {
    let markdown = html_to_markdown_with_base_url(
        r#"<img src="a b.png" alt="图片">"#,
        "https://jw.nju.edu.cn/ggtz/page.html",
    )
    .unwrap();

    assert!(
        markdown.contains("![图片](<https://jw.nju.edu.cn/ggtz/a%20b.png>)"),
        "{markdown}"
    );
}

#[tokio::test]
async fn jw_page_contains_absolutized_image_url() {
    let client = ClientBuilder::new(reqwest::Client::new()).build();
    let markdown = read_html_page(
        &client,
        "https://jw.nju.edu.cn/bd/07/c26263a834823/page.htm",
    )
    .await
    .unwrap();

    assert!(
        markdown.contains("https://jw.nju.edu.cn/_upload/article/images/f9/8b/a448dbf84d1393c91f184df2ada5/7a4a35d8-2f41-4e65-ba98-999d97a2433c.png"),
        "{markdown}"
    );
}

#[tokio::test]
async fn jw_page_contains_absolutized_pdf_url() {
    let client = ClientBuilder::new(reqwest::Client::new()).build();
    let markdown = read_html_page(
        &client,
        "https://jw.nju.edu.cn/b1/37/c26263a831799/page.htm",
    )
    .await
    .unwrap();

    assert!(
        markdown.contains("https://jw.nju.edu.cn/_upload/article/files/46/4f/3c03ddbe4a819cf5a89a5da308c8/cc499c20-f59f-4fca-9853-5bfc3b442ebc.pdf"),
        "{markdown}"
    );
}
