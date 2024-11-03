// Rustコード (main.rs)

use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use yew::prelude::*;
use gloo_net::http::{Request};
use gloo_timers::callback::Timeout; // gloo_timersからTimeoutをインポート
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlElement};
use anyhow::Error;
use chrono::{DateTime, Local};
use serde::{Deserialize};
use serde_json::json; // JSONエンコード用にserde_jsonをインポート
use pulldown_cmark::{Parser, html};
use yew::virtual_dom::VNode;

// Convert markdown to html
fn markdown_to_html(content: &str) -> Html {
    let parser = Parser::new(content);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // HTML文字列をDOMノードに変換し、YewのVNodeとして返す
    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("div").unwrap();
    div.set_inner_html(&html_output);
    VNode::VRef(div.into())
}

// From the backend 
#[derive(Debug, Deserialize)]  // Deserializeを追加
struct EntryResponse {
    id: String,           
    content: String,      
    created_at: String,   
}
impl EntryResponse {
    fn to_entry(self) -> Option<Entry> {
        if let Ok(datetime) = DateTime::parse_from_rfc3339(&self.created_at) {
            Some(Entry {
                id: self.id,
                log: self.content,
                timestamp: datetime.with_timezone(&Local),
            })
        } else {
            None
        }
    }
}


pub struct Entry {
    id: String,
    log: String,
    timestamp: DateTime<Local>,
}
impl Entry {
    fn new(id:String, log:String, timestamp:DateTime<Local>) -> Self {
        Self{id:id, log:log, timestamp:timestamp}
    }
}

pub struct Model {
    entry:   String,
    entries: Vec<Entry>,
    limit:   i64,
    offset:  i64,
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

pub enum Msg {
    UpdateEntry(String),
    AddEntry,
    GetEntries(i64, i64),
    LoadMoreEntries,
    ReceiveResponse(Result<Vec<Entry>, Error>),
    ReceiveLatestEntry(Entry)
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let default_limit = 20;

        let link = ctx.link().clone();
        link.send_message(Msg::GetEntries(default_limit, 0));
        let instance = Self {
            entry: String::new(),
            entries: vec![],
            limit: default_limit,
            offset: 0,
            loading: false,
            content_ref: NodeRef::default(), // NodeRefの初期化
        };

        // JavaScript側でアクセスできるように関数を登録
        register_update_and_add_entry_callback(ctx.link().clone());

        instance
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateEntry(val) => {
                self.entry = val;
                true
            }

            Msg::AddEntry => {
                let link = ctx.link().clone();
                if self.entry.is_empty() {
                    link.send_message(Msg::ReceiveResponse(Err(anyhow::anyhow!("Entry is empty"))));
                    return false;
                }
                let entry_content = self.entry.clone();
                let body = json!({"content": entry_content}).to_string(); // can contain \n

                spawn_local(async move {
                    if let Ok(response) = Request::post("http://127.0.0.1:8080/add_entry")
                        .header("Content-Type", "application/json")
                        .body(body)
                        .send()
                        .await
                    {
                        if response.ok() {
                            //link.send_message(Msg::GetEntries(limit, offset));
                            link.send_message(Msg::GetEntries(1, 0));
                        } else {
                            link.send_message(Msg::ReceiveResponse(Err(anyhow::anyhow!("Request failed"))));
                        }
                    } else {
                        link.send_message(Msg::ReceiveResponse(Err(anyhow::anyhow!("Request failed."))));
                    }
                });
                self.entry.clear();
                true
            }

            Msg::GetEntries(limit, offset) => {
                let link = ctx.link().clone();
                self.loading = true;

                spawn_local(async move {
                    if let Ok(response) = Request::get(&format!("http://127.0.0.1:8080/get_entries?limit={}&offset={}", limit, offset))
                        .send()
                        .await
                    {
                        if let Ok(json) = response.json::<Vec<EntryResponse>>().await {
                            let entries: Vec<Entry> = 
                                json.into_iter().filter_map(|entry_response| {
                                    entry_response.to_entry()
                                }).collect();
                            
                            // Only taking the newly entered entry
                            if limit == 1 && offset == 0 {
                                if let Some(new_entry) = entries.into_iter().next() {
                                    link.send_message(Msg::ReceiveLatestEntry(new_entry));
                                }
                            } else {
                                link.send_message(Msg::ReceiveResponse(Ok(entries)));
                            }
                        } else {
                            link.send_message(Msg::ReceiveResponse(Err(anyhow::anyhow!("Failed to parse request as text"))));
                        }
                    } else {
                        link.send_message(Msg::ReceiveResponse(Err(anyhow::anyhow!("Request failed."))));
                    }
                });
                false
            }
            
            Msg::ReceiveLatestEntry(new_entry) => {
                // 新しいエントリーをリストの先頭に追加
                self.entries.push(new_entry);
                self.offset += 1;
                self.loading = false;

                // Force to scroll down
                self.scroll_to_position(0, true, 50);

                true
            }            
            
            Msg::LoadMoreEntries => {
                if ! self.loading {
                    ctx.link().send_message(Msg::GetEntries(self.limit, self.offset));
                }
                false
            }   

            Msg::ReceiveResponse(response) => {
                match response {

                    Ok(entries) => {
                        // Add the loaded entried
                        self.offset += entries.len() as i64;
                        entries.into_iter().for_each(|entry| self.entries.insert(0, entry));
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

    fn view(&self, ctx: &Context<Self>) -> Html {
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
                                let entry_date = entry.timestamp.with_timezone(&Local).date();
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
                                            <span class="log-text">{markdown_to_html(&entry.log)}</span>
                                        </li>
                                    </>
                                }
                            })
                        }
                    </ul>
                </div>
                <div class="resize-divider"></div>
                <footer class="footer">
                    <textarea
                        value={self.entry.clone()}
                        class="input-box"
                        placeholder="Enter text here..."
                    />
                </footer>
            </div>
        }
    }    
    
}



// JavaScript用の関数を登録する
fn register_update_and_add_entry_callback(link: yew::html::Scope<Model>) {
    // クロージャを作成してJavaScript側に公開
    let callback = Closure::wrap(Box::new(move |content: String| {
        link.send_message(Msg::UpdateEntry(content.clone())); // UpdateEntryを送信
        link.send_message(Msg::AddEntry); // AddEntryを送信
    }) as Box<dyn Fn(String)>);

    // JavaScript からこのクロージャを呼び出せるように、`send_update_and_add_entry` として登録
    let global = js_sys::global();
    js_sys::Reflect::set(
        &global,
        &JsValue::from_str("send_update_and_add_entry"),
        callback.as_ref().unchecked_ref(),
    )
    .expect("Failed to register `send_update_and_add_entry`");

    callback.forget(); // メモリ解放を防ぐため、忘却する
}


fn main() {
    yew::Renderer::<Model>::new().render();
}
