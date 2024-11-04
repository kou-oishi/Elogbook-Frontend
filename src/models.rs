use chrono::{DateTime, Local};
use serde::Deserialize;

// From the backend
#[derive(Debug, Deserialize)]
pub struct Attachment {
    pub id: u32,
    pub mime: String,
    pub download_url: String,
    pub original_name: String,
}
pub type Attachments = Vec<Attachment>;

#[derive(Debug, Deserialize)] // Deserializeを追加
pub struct EntryResponse {
    pub id: String,
    pub content: String,
    pub created_at: String,
    pub attachments: Attachments,
}
impl EntryResponse {
    pub fn to_entry(self) -> Option<Entry> {
        if let Ok(datetime) = DateTime::parse_from_rfc3339(&self.created_at) {
            Some(Entry {
                id: self.id,
                log: self.content,
                timestamp: datetime.with_timezone(&Local),
                attachments: self.attachments,
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub id: String,
    pub log: String,
    pub timestamp: DateTime<Local>,
    pub attachments: Attachments,
}
impl Entry {
    pub fn new(
        id: String,
        log: String,
        timestamp: DateTime<Local>,
        attachments: Attachments,
    ) -> Self {
        Self {
            id: id,
            log: log,
            timestamp: timestamp,
            attachments: attachments,
        }
    }
}
