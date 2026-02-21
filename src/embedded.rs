use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "public/"]
#[exclude = "assets/*"]
pub struct Assets;

pub fn get_html_content(path: &str) -> Option<String> {
    let asset = Assets::get(path)?;
    String::from_utf8(asset.data.to_vec()).ok()
}

pub fn serve_html_file(path: &str) -> axum::response::Html<String> {
    match get_html_content(path) {
        Some(content) => axum::response::Html(content),
        None => axum::response::Html(format!(
            r#"<!DOCTYPE html>
<html>
<head><title>404</title></head>
<body><h1>404 - File not found</h1><p>{}</p></body>
</html>"#,
            path
        )),
    }
}