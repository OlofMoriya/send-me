use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Note {
    pub id: usize,
    pub text: String,
    pub date: DateTime<Local>,
    pub reciever: String,
    pub sender: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NoteDto {
    pub text: String,
    pub sender: String,
    pub reciever: Option<String>,
    pub persist: bool,
}

