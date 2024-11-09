use anyhow::Error;
use chrono::Local;
use gloo_net::http::Request;
use gloo_timers::callback::Timeout;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{FormData, HtmlElement};
use yew::prelude::*;

mod models;
use models::*;

mod render;
use render::*;

pub struct Model {
    entries: Vec<Entry>,
    limit: i64,
    offset: i64,
    loading: bool,
    content_ref: NodeRef, // NodeRef追加
}

impl Model {
    // レンダー完了後にスクロール位置を設定する関数
    fn scroll_to_position(&self, offset: i32, from_bottom: bool, waiting_time: u32) {
        let content_ref = self.content_ref.clone();

        // 50ミリ秒待機してからスクロール位置を設定
        Timeout::new(waiting_time, move || {
            if let Some(content) = content_ref.cast::<HtmlElement>() {
                if from_bottom {
                    // 絶対的に最下部から offset だけ上にずらす
                    content.set_scroll_top(content.scroll_height() - offset);
                } else {
                    // 上部から offset だけ下にずらす
                    content.set_scroll_top(offset);
                }
            }
        })
        .forget(); // コールバックを忘却
    }
}

// Message handlers for the model
pub enum Msg {
    AddEntry(String, Vec<web_sys::File>),
    GetEntries(i64, i64),
    LoadMoreEntries,
    ReceiveResponse(Result<Vec<Entry>, Error>),
    ReceiveLatestEntry(Entry),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    /////////////////////////////////////////////////////////////////////////////////////////////
    /// create
    /////////////////////////////////////////////////////////////////////////////////////////////
    fn create(ctx: &Context<Self>) -> Self {
        let default_limit = 20;

        let link = ctx.link().clone();
        link.send_message(Msg::GetEntries(default_limit, 0));
        let instance = Self {
            entries: vec![],
            limit: default_limit,
            offset: 0,
            loading: false,
            content_ref: NodeRef::default(), // NodeRefの初期化
        };

        // JavaScript側でアクセスできるように関数を登録
        register_entry_callback(ctx.link().clone());

        instance
    }

    /////////////////////////////////////////////////////////////////////////////////////////////
    /// update
    /////////////////////////////////////////////////////////////////////////////////////////////
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            // ---------------------------------------------------------------------------
            // Message: AddEntry
            // ---------------------------------------------------------------------------
            Msg::AddEntry(content, attachments) => {
                let link = ctx.link().clone();
                if content.is_empty() && attachments.is_empty() {
                    link.send_message(Msg::ReceiveResponse(Err(anyhow::anyhow!("Empty entry"))));
                    return false;
                }
                // Compile the data into fromdata
                let form_data = FormData::new().unwrap();
                // Content
                form_data.append_with_str("content", &content).unwrap();
                // Attachments
                for file in attachments {
                    form_data
                        .append_with_blob_and_filename("file", &file, &file.name())
                        .unwrap();
                }

                spawn_local(async move {
                    let request_init = web_sys::RequestInit::new();
                    request_init.set_method("POST");
                    request_init.set_body(&JsValue::from(form_data));

                    let request = web_sys::Request::new_with_str_and_init(
                        "http://127.0.0.1:8080/add_entry",
                        &request_init,
                    )
                    .unwrap();

                    let window = web_sys::window().unwrap();
                    let fetch_promise = window.fetch_with_request(&request);

                    match wasm_bindgen_futures::JsFuture::from(fetch_promise).await {
                        Ok(response) => {
                            let response: web_sys::Response = response.dyn_into().unwrap();
                            if response.ok() {
                                link.send_message(Msg::GetEntries(1, 0));
                            } else {
                                link.send_message(Msg::ReceiveResponse(Err(anyhow::anyhow!(
                                    "Request failed."
                                ))));
                            }
                        }
                        Err(err) => {
                            link.send_message(Msg::ReceiveResponse(Err(anyhow::anyhow!(format!(
                                "Request failed: {:?}",
                                err
                            )))));
                        }
                    }
                });
                true
            }

            // ---------------------------------------------------------------------------
            // Message: GetEntries
            // ---------------------------------------------------------------------------
            Msg::GetEntries(limit, offset) => {
                let link = ctx.link().clone();
                self.loading = true;

                spawn_local(async move {
                    if let Ok(response) = Request::get(&format!(
                        "http://127.0.0.1:8080/get_entries?limit={}&offset={}",
                        limit, offset
                    ))
                    .send()
                    .await
                    {
                        if let Ok(json) = response.json::<Vec<EntryResponse>>().await {
                            let entries: Vec<Entry> = json
                                .into_iter()
                                .filter_map(|entry_response| entry_response.to_entry())
                                .collect();

                            // Only taking the newly entered entry
                            if limit == 1 && offset == 0 {
                                if let Some(new_entry) = entries.into_iter().next() {
                                    link.send_message(Msg::ReceiveLatestEntry(new_entry));
                                }
                            } else {
                                link.send_message(Msg::ReceiveResponse(Ok(entries)));
                            }
                        } else {
                            link.send_message(Msg::ReceiveResponse(Err(anyhow::anyhow!(
                                "Failed to parse request as text"
                            ))));
                        }
                    } else {
                        link.send_message(Msg::ReceiveResponse(Err(anyhow::anyhow!(
                            "Request failed."
                        ))));
                    }
                });
                false
            }

            // ---------------------------------------------------------------------------
            // ReceiveLatestEntry
            // ---------------------------------------------------------------------------
            Msg::ReceiveLatestEntry(new_entry) => {
                // 新しいエントリーをリストの先頭に追加
                self.entries.push(new_entry);
                self.offset += 1;
                self.loading = false;

                // Force to scroll down
                self.scroll_to_position(0, true, 50);
                true
            }

            // ---------------------------------------------------------------------------
            // Message: LoadMoreEntries
            // ---------------------------------------------------------------------------
            Msg::LoadMoreEntries => {
                if !self.loading {
                    ctx.link()
                        .send_message(Msg::GetEntries(self.limit, self.offset));
                }
                false
            }

            // ---------------------------------------------------------------------------
            // Message: ReceiveResponse
            // ---------------------------------------------------------------------------
            Msg::ReceiveResponse(response) => {
                match response {
                    Ok(entries) => {
                        // Add the loaded entried
                        self.offset += entries.len() as i64;
                        entries
                            .into_iter()
                            .for_each(|entry| self.entries.insert(0, entry));
                        self.loading = false;

                        self.scroll_to_position(10, false, 50);

                        true
                    }
                    Err(err) => {
                        web_sys::console::log_1(&format!("Error! {:?}", err).into());
                        false
                    }
                }
            }
        }
    }

    /////////////////////////////////////////////////////////////////////////////////////////////
    /// rendered
    /////////////////////////////////////////////////////////////////////////////////////////////
    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            // Force to scroll down
            self.scroll_to_position(0, true, 300);

            // スクロールイベントリスナーを追加
            let link = ctx.link().clone();
            let content_ref = self.content_ref.clone();
            let callback = Closure::<dyn Fn()>::new(move || {
                if let Some(content) = content_ref.cast::<HtmlElement>() {
                    if content.scroll_top() == 0 {
                        link.send_message(Msg::LoadMoreEntries);
                    }
                }
            });

            // スクロールイベントをリッスン
            if let Some(content) = self.content_ref.cast::<HtmlElement>() {
                content
                    .add_event_listener_with_callback("scroll", callback.as_ref().unchecked_ref())
                    .unwrap();
            }

            // イベントハンドラを保持
            callback.forget();
        }
    }

    /////////////////////////////////////////////////////////////////////////////////////////////
    /// view
    /////////////////////////////////////////////////////////////////////////////////////////////
    fn view(&self, _ctx: &Context<Self>) -> Html {
        let mut last_date = None;

        html! {
            <div class="container">
                <header class="header">
                    <h1>{"Elogbook Entries"}</h1>
                </header>
                <div ref={self.content_ref.clone()} id="content" class="content">
                    <ul class="entries-list">
                        {
                            for self.entries.iter().map(|entry| {
                                let entry_date = entry.timestamp.with_timezone(&Local).date_naive();
                                let show_date = match last_date {
                                    Some(last) if last == entry_date => false,
                                    _ => {
                                        last_date = Some(entry_date);
                                        true
                                    }
                                };
                                html! {
                                    <>
                                        if show_date {
                                            <li class="entry-date">
                                                { entry_date.format("%Y-%m-%d").to_string() }
                                            </li>
                                        }
                                        <li class="entry-item">
                                            <span class="timestamp">
                                                { entry.timestamp.with_timezone(&Local).format("%H:%M:%S").to_string() }
                                            </span>
                                            <span class="log-text">{markdown_to_html(entry)}</span>
                                        </li>
                                    </>
                                }
                            })
                        }
                    </ul>
                </div>
                <div id="file-previews" class="file-previews"></div>
                <div class="resize-divider"></div>
                <footer class="footer">
                    <textarea
                        value=""
                        class="input-box"
                        placeholder="Enter text here..."
                    />
                </footer>
            </div>
        }
    }
}

// Interface to Java Script
fn register_entry_callback(link: yew::html::Scope<Model>) {
    // クロージャを作成してJavaScript側に公開
    let callback = Closure::wrap(Box::new(move |content: JsValue, array: JsValue| {
        let content_str = content.as_string().unwrap_or_default();
        let files = js_sys::Array::from(&array);
        let attachments: Vec<web_sys::File> = files
            .iter()
            .map(|f| f.dyn_into::<web_sys::File>().unwrap())
            .collect();

        link.send_message(Msg::AddEntry(content_str.clone(), attachments)); // Send AddEntry
    }) as Box<dyn Fn(JsValue, JsValue)>);

    let global = js_sys::global();
    js_sys::Reflect::set(
        &global,
        &JsValue::from_str("send_add_entry"),
        callback.as_ref().unchecked_ref(),
    )
    .expect("Failed to register `send_add_entry`");

    callback.forget();
}

fn main() {
    yew::Renderer::<Model>::new().render();
}
