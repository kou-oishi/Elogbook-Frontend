use pulldown_cmark::{html, Parser};
use yew::prelude::*;
use yew::virtual_dom::VNode;

use crate::models::*;

// Convert markdown to html
pub fn markdown_to_html(content: &str) -> Html {
    let parser = Parser::new(content);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // HTML文字列をDOMノードに変換し、YewのVNodeとして返す
    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("div").unwrap();
    div.set_inner_html(&html_output);
    VNode::VRef(div.into())
}

pub fn expand_attachment_html(attachment: &Attachment) -> String {
    /*
    match attachment.mime.as_str() {
        "image/png" | "image/jpeg" | "image/gif" => {
            format!(
                "<img src='/attachments/{}' alt='Attachment Image' />",
                attachment.saved_path
            )
        }
        "text/plain" => {
            format!(
                "<div class='text-attachment' style='max-height: 200px; overflow-y: auto;'>{}</div>",
                html_escape::encode_text(&attachment.original_name)
            )
        }
        "application/octet-stream" => {
            format!(
                "<div class='code-attachment' style='max-height: 200px; overflow-y: auto;'><pre>{}</pre></div>",
                html_escape::encode_text(&attachment.original_name)
            )
        }
        _ => String::new(),
    }
    */
    String::new()
}
