use std::fs::{OpenOptions, self};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use actix_web::{get, post, web, App, HttpServer, Responder};
use chrono::{DateTime, Local};
use serde::Serialize;
use std::io::Write;

#[derive(Serialize, Clone)]
struct Note {
    id: usize,
    text: String, 
    date: DateTime<Local>,
    user: String, 
}

struct AppState {
    memory: Mutex<Vec<Note>>,
}

fn load_data() -> Vec<Note> {
    let input = include_str!("../backup");
    let lines = input.split("\n§!§\n");
    return lines.filter_map(|l| { 
        if l == "" {return None;}
        let values:Vec<&str> = l.split("§§§").collect();
        let id = values[0].parse::<usize>().unwrap_or(0); 
        let text = values[1];
        let date = DateTime::parse_from_str(values[2], "%Y-%m-%d %H:%M:%S %z").unwrap();
        let user = values[3];
        return Some(Note{
            id,
            text: text.to_string(),
            date:date.with_timezone(&Local),
            user: user.to_string()
        });
    }).collect();
}

fn append_date(note:&Note) {

    let data = vec!(note.id.to_string(), 
                    note.text.clone(), 
                    format!("{}",note.date.format("%Y-%m-%d %H:%M:%S %z")),
                    note.user.clone()
                    ); 
    let to_save:String = data.join("§§§");

    let mut fappend = fs::OpenOptions::new()
                                     .append(true)
                                     .open("./backup")
                                     .unwrap();
    write!(fappend, "{}\n§!§\n", to_save).unwrap();
}

static COUNTER:AtomicUsize = AtomicUsize::new(0);

fn get_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[get("/")]
async fn list(data: web::Data<AppState>) -> impl Responder {
    let list = data.memory.lock().unwrap();
    return web::Json(list.clone());
}

#[post("/add")]
async fn add(data: web::Data<AppState>, req_body: String) -> impl Responder {
    let note = Note{
        id: get_id(),
        text: req_body.clone(),
        date: Local::now(),
        user: "-".to_string()
    };

    data.memory.lock().unwrap().push(note.clone());
    append_date(&note);

    return web::Json(note);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let notes = load_data();
    let max = notes.iter().max_by_key(|x| x.id);
    let max_id = match max {
        Some(v) => v.id,
        None => 0
    };

    COUNTER.store(max_id + 1, Ordering::Relaxed);

    let state = web::Data::new(AppState {
        memory: Mutex::new(notes),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(list)
            .service(add)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
