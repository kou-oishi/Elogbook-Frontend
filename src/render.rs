use crate::models::*;
use pulldown_cmark::{html, Parser};
use yew::prelude::*;
use yew::virtual_dom::VNode;

// Convert markdown to html
pub fn markdown_to_html(entry: &Entry) -> Html {
    let log_with_attachments = parse_log_text(&entry.log, &entry.attachments);
    let parser = Parser::new(&log_with_attachments);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // HTML文字列をDOMノードに変換し、YewのVNodeとして返す
    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("div").unwrap();
    div.set_inner_html(&html_output);
    VNode::VRef(div.into())
}

fn parse_log_text(log_text: &str, attachments: &[Attachment]) -> String {
    let mut result = String::new();
    let mut chars = log_text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some('%') = chars.peek() {
                result.push('%');
                chars.next();
            } else {
                result.push(c);
            }
        } else if c == '%' {
            let mut id_str = String::new();
            while let Some(&next) = chars.peek() {
                if next.is_numeric() {
                    id_str.push(next);
                    chars.next();
                } else {
                    break;
                }
            }

            if let Ok(id) = id_str.parse::<u32>() {
                if let Some(attachment) = attachments.iter().find(|att| att.id == id) {
                    result.push_str(&expand_attachment_html(attachment));
                } else {
                    result.push_str(&format!("%{}", id));
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn expand_attachment_html(attachment: &Attachment) -> String {
    use html_escape::encode_text;

    let path = format!("http://127.0.0.1:8080/{}", attachment.download_url);

    match attachment.mime.as_str() {
        "image/png" | "image/jpeg" | "image/gif" => {
            format!(
                "<img src='{}' alt='{}' />",
                path,
                encode_text(&attachment.original_name)
            )
        }

        /*
        "text/plain" => match ureq::get(&path).call() {
            Ok(response) => {
                if let Ok(content) = response.into_string() {
                    format!(
                            "<div class='text-attachment' style='max-height: 200px; overflow-y: auto;'><pre>{}</pre></div>",
                            encode_text(&content)
                        )
                } else {
                    format!(
                        "<p>Failed to load content: {}</p>",
                        encode_text(&attachment.original_name)
                    )
                }
            }
            Err(_) => format!(
                "<p>Failed to fetch the content: {}</p>",
                encode_text(&attachment.original_name)
            ),
        },
        */
        "application/octet-stream" => {
            format!(
                "<div class='code-attachment' style='max-height: 200px; overflow-y: auto;'><pre>{}</pre></div>",
                html_escape::encode_text(&path)
            )
        }
        _ => String::new(),
    }
}
