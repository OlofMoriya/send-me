use std::fs::{ self };
use std::io::Write;

use chrono::{Local, DateTime};
use crate::model::Note;

pub fn load_data() -> Vec<Note> {
    let input = include_str!("../backup");
    let lines = input.split("\n§!§\n");
    return lines
        .filter_map(|l| {
            if l == "" {
                return None;
            }
            let values: Vec<&str> = l.split("§§§").collect();
            let id = values[0].parse::<usize>().unwrap_or(0);
            let text = values[1];
            let date = DateTime::parse_from_str(values[2], "%Y-%m-%d %H:%M:%S %z").unwrap();
            let sender = values[3];
            
            let reciever = match values.len() {
                0 ..= 4 => sender,
                _ => values[4]
            };

            return Some(Note {
                id,
                text: text.to_string(),
                date: date.with_timezone(&Local),
                sender: sender.to_string(),
                reciever: reciever.to_string(),
            });
        })
        .collect();
}

pub fn append_note(note: &Note) {
    let data = vec![
        note.id.to_string(),
        note.text.clone(),
        format!("{}", note.date.format("%Y-%m-%d %H:%M:%S %z")),
        note.sender.clone(),
        note.reciever.clone(),
    ];
    let to_save: String = data.join("§§§");

    let mut fappend = fs::OpenOptions::new()
        .append(true)
        .open("./backup")
        .unwrap();
    write!(fappend, "{}\n§!§\n", to_save).unwrap();
}
