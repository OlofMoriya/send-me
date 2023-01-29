mod model;
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::http::header::ContentType;
use actix_web::web::Query;
use model::Note;
mod io;
use io::{load_data, append_note};
use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpRequest, HttpServer, ResponseError, HttpResponse};
use chrono::Local;
use serde::{Serialize, Deserialize};
use std::env;
use std::fmt::Display;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::model::NoteDto;

struct AppState {
    memory: Mutex<Vec<Note>>,
    api_key: String
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn get_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn check_api_key(data: &web::Data<AppState>, request: &HttpRequest) -> bool {
    let headers = request.headers();
    let api_key_header = match headers.get("api_key") {
        Some(v) => v.to_str().ok().unwrap().to_string(),
        None => return false
    };

    if api_key_header != data.api_key {
        return false;
    }

    return true;
}

#[derive(Debug, Serialize)]
struct ErrBadApiKey {
    err: &'static str
}

impl Display for ErrBadApiKey {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:?}", self)
   }
}

impl ResponseError for ErrBadApiKey {
    fn status_code(&self) -> StatusCode {
        return StatusCode::BAD_REQUEST;
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let body = serde_json::to_string(&self).unwrap();
        let res = HttpResponse::new(self.status_code());
        return res.set_body(BoxBody::new(body));
    }
}

#[get("/")]
async fn list(data: web::Data<AppState>, request: HttpRequest) -> HttpResponse
{
    if !check_api_key(&data, &request) {
        return HttpResponse::BadRequest().body("Empty or Incorrect api key")
    }

    let list = data.memory.lock().unwrap();
    let response_body = serde_json::to_string(&list.clone()).unwrap();
    let response = HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_body);
    return response;
}

#[derive(Deserialize)]
struct LatestQuery {
    user: String,
}

#[get("/latest")]
async fn latest(latest_query: Query<LatestQuery>, data: web::Data<AppState>, request: HttpRequest) -> HttpResponse {
    if !check_api_key(&data, &request) {
        return HttpResponse::BadRequest().body("Empty or Incorrect api key")
    }

    let other_list = data.memory.lock().unwrap();
    let last = other_list
        .clone()
        .into_iter()
        .filter(|c| c.sender == latest_query.user || c.reciever == latest_query.user)
        .last();


    let response_body = serde_json::to_string(&last).unwrap();
    let response = HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_body);
    return response;
}

#[post("/add")]
async fn add(data: web::Data<AppState>, req_body: String, request: HttpRequest) -> HttpResponse {
    if !check_api_key(&data, &request) {
        return HttpResponse::BadRequest().body("Empty or Incorrect api key")
    }

    let note_dto = serde_json::from_str::<NoteDto>(req_body.as_str());

    let headers = request.headers();
    let user_header = match headers.get("user") {
        Some(v) => v.to_str().ok().unwrap().to_string(),
        None => "-".to_string(),
    };

    let mut persist = true;
    let note: Option<Note>;

    match note_dto {
        Ok(note_dto) => {
            note = Some(Note {
                id: get_id(),
                text: note_dto.text,
                date: Local::now(),
                sender: note_dto.sender.clone(),
                reciever: match note_dto.reciever {
                    Some(v) => v,
                    None => note_dto.sender
                }
            });
            persist = note_dto.persist;
        }
        Err(_) => {
            note = Some(Note {
                id: get_id(),
                text: req_body.clone(),
                date: Local::now(),
                sender: user_header.clone(),
                reciever: user_header,
            })
        }
    };

    let note = note.expect("There will always be a note");

    if persist {
        append_note(&note);
    }
    data.memory.lock().unwrap().push(note.clone());

    let response_body = serde_json::to_string(&note).unwrap();
    let response = HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_body);
    return response;
}

fn read_config() -> Option<String> {
    let api_key = env::var("SEND_ME_KEY");
    return api_key.ok();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let notes = load_data();
    let api_key = match read_config(){
        Some(v) => v,
        None => "There should be a key in the environment".to_string()
    };

    let max = notes.iter().max_by_key(|x| x.id);
    let max_id = match max {
        Some(v) => v.id,
        None => 0,
    };

    COUNTER.store(max_id + 1, Ordering::Relaxed);

    let state = web::Data::new(AppState {
        memory: Mutex::new(notes),
        api_key
    });

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .app_data(state.clone())
            .wrap(cors)
            .service(list)
            .service(latest)
            .service(add)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
