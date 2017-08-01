#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate error_chain;

extern crate itertools;

extern crate hyper;
extern crate futures;
extern crate tokio_core;

use futures::{Future, Stream};
use itertools::Itertools;
use hyper::{Chunk, Client, Method, Request};
use hyper::client::HttpConnector;
pub use tokio_core::reactor::Core;

mod errors {
    error_chain! {
        foreign_links {
            Hyper(::hyper::Error);
            Json(::serde_json::Error);
        }
    }
}

pub use errors::{Error,Result};

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Song {
    pub id: u32,
    pub title: String,
    pub artist: String,
    pub language: String,
    pub play_count: u32,
    pub added_on: String,
    pub favorite: bool
}

#[derive(Deserialize, Debug, Default)]
pub struct Play {
    pub song_id: u32,
    pub play_id: String,
    pub title: String,
    pub artist: String,
    pub location: String,
    pub business_date: String,
    pub enqueue_time: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub duration: u32,
    pub position: u32,
    pub favorite: bool,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Default)]
pub struct LoginResponse {
    session: String,
    email: String,
    handle: String,
    color: String,
    hide_handle_in_queue: bool
}

#[derive(Deserialize, Debug, Default)]
pub struct QueueResponse {
    index: u32,
    song_id: u32,
    play_id: String,
    title: String,
    artist: String,
    duration: u32
}

#[derive(Deserialize, Debug, Default)]
pub struct FavoritesResponse {
    page: u32,
    per_page: u32,
    total_pages: u32,
    total_entries: u32,
    songs: Vec<Song>
}

#[derive(Deserialize, Debug, Default)]
pub struct HistoryResponse {
    page: u32,
    per_page: u32,
    total_pages: u32,
    total_entries: u32,
    plays: Vec<Play>
}

pub struct Voicebox {
    code: Option<String>,
    client: Client<HttpConnector>,
    session: String,
}

pub type BoxFuture<T> = Box<Future<Item=T, Error=Error>>;

impl Voicebox {
    pub fn new(room_code: Option<String>, core: &mut Core) -> Voicebox {
        let mut handle = core.handle();
        Voicebox { code: room_code, client: Client::new(&mut handle), session: String::new() }
    }

    fn request<T: 'static + serde::de::DeserializeOwned> (&mut self, method: Method, endpoint: &str, params: Vec<(&str, &str)>) -> BoxFuture<T> {
        let query = params.into_iter().map(|p| format!("{}={}", p.0, p.1)).join("&");
        let uri_str = format!("http://voiceboxpdx.com/api/v1/{}.json?{}", endpoint, query);

        // We allow this to panic, since we just built the uri string
        let uri = uri_str.parse().unwrap();
        let req = Request::new(method, uri);

        Box::new(self.client.request(req).from_err().and_then(|res| {
            res.body().concat2().from_err().and_then(move |body: Chunk| {
                serde_json::from_slice(&body).map_err(|e| Error::from(e))
            })
        }).from_err())
    }

    pub fn login(&mut self, email: &str) -> BoxFuture<LoginResponse> {
        let params = vec![("email", email)];
        self.request(Method::Post, "login", params)
    }

    pub fn popup(&mut self, msg: &str) -> BoxFuture<()> {
        // TODO: don't clone when we can properly do a partial
        // borrow of this struct
        let session = self.session.clone();
        let code = if self.code.is_some() {
            self.code.as_ref().unwrap().to_owned()
        } else {
            return Box::new(futures::future::result(Err("Missing room code!".into())));
        };
        let params: Vec<(&str, &str)> = vec![("session", &session),
                                             ("room_code", &code),
                                             ("text", msg)];
        self.request(Method::Post, "login", params)
    }

    pub fn set_handle(&mut self, handle: &str) -> BoxFuture<()> {
        // TODO: don't clone when we can properly do a partial
        // borrow of this struct
        let session = self.session.clone();
        let params: Vec<(&str, &str)> = vec![("session", &session),
                                             ("handle", handle)];
        self.request(Method::Put, "profile", params)
    }

    pub fn enqueue_song(&mut self, id: &str) -> BoxFuture<QueueResponse> {
        // TODO: don't clone when we can properly do a partial
        // borrow of this struct
        let session = self.session.clone();
        let code = if self.code.is_some() {
            self.code.as_ref().unwrap().to_owned()
        } else {
            return Box::new(futures::future::result(Err("Missing room code!".into())));
        };
        let params: Vec<(&str, &str)> = vec![("session", &session),
                                             ("room_code", &code),
                                             ("song_id", id)];
        self.request(Method::Post, "queue", params)
    }

    pub fn delete_song(&mut self, id: &str) -> BoxFuture<()> {
        // TODO: don't clone when we can properly do a partial
        // borrow of this struct
        let session = self.session.clone();
        let code = if self.code.is_some() {
            self.code.as_ref().unwrap().to_owned()
        } else {
            return Box::new(futures::future::result(Err("Missing room code!".into())));
        };
        let params: Vec<(&str, &str)> = vec![("session", &session),
                                             ("room_code", &code),
                                             ("from", id)];
        self.request(Method::Delete, "queue", params)
    }

    pub fn favorites(&mut self, page: u32, per_page: u32) -> BoxFuture<FavoritesResponse> {
        // TODO: don't clone when we can properly do a partial
        // borrow of this struct
        let session = self.session.clone();
        let page_as_str = format!("{}", page);
        let per_page_as_str = format!("{}", per_page);
        let params: Vec<(&str, &str)> = vec![("session", &session),
                                             ("page", &page_as_str),
                                             ("per_page", &per_page_as_str)];
        self.request(Method::Get, "songs/favorites", params)
    }

    pub fn history(&mut self, page: u32, per_page: u32) -> BoxFuture<HistoryResponse> {
        // TODO: don't clone when we can properly do a partial
        // borrow of this struct
        let session = self.session.clone();
        let page_as_str = format!("{}", page);
        let per_page_as_str = format!("{}", per_page);
        let params: Vec<(&str, &str)> = vec![("session", &session),
                                             ("page", &page_as_str),
                                             ("per_page", &per_page_as_str)];
        self.request(Method::Get, "plays/history", params)
    }
}
