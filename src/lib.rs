#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate itertools;

extern crate hyper;
extern crate futures;
extern crate tokio_core;

use futures::{Future, Stream};
use itertools::Itertools;
use hyper::{Chunk, Client, Method, Request};
use hyper::client::HttpConnector;
pub use tokio_core::reactor::{Core, Handle};

#[derive(Deserialize, Debug, Default)]
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
struct LoginResponse {
    session: String,
    email: String,
    handle: String,
    color: String,
    hide_handle_in_queue: bool
}

#[derive(Deserialize, Debug, Default)]
struct QueueResponse {
    index: u32,
    song_id: u32,
    play_id: String,
    title: String,
    artist: String,
    duration: u32
}

#[derive(Deserialize, Debug, Default)]
struct FavoritesResponse {
    page: u32,
    per_page: u32,
    total_pages: u32,
    total_entries: u32,
    songs: Vec<Song>
}

pub struct Voicebox<'a> {
    core: &'a mut Core,
    code: String,
    client: Client<HttpConnector>,
    session: String,
}

impl<'a> Voicebox<'a> {
    pub fn new(room_code: &'a str, core: &'a mut Core, handle: &'a mut Handle) -> Voicebox<'a> {
        Voicebox { core: core, code: room_code.to_owned(), client: Client::new(handle), session: String::new() }
    }

    fn request<T: serde::de::DeserializeOwned> (&mut self, method: Method, endpoint: &str, params: Vec<(&str, &str)>) -> T {
        let query = params.into_iter().map(|p| format!("{}={}", p.0, p.1)).join("&");
        let uri_str = format!("http://voiceboxpdx.com/api/v1/{}.json?{}", endpoint, query);
        let uri = uri_str.parse().unwrap();
        let req = Request::new(method, uri);

        let work = self.client.request(req).and_then(|res| {
            res.body().concat2().and_then(move |body: Chunk| {
                Ok(serde_json::from_slice(&body).unwrap())
            })
        });
        self.core.run(work).unwrap()
    }

    pub fn login(&mut self, email: &str) -> String {
        let params = vec![("email", email)];
        let resp: LoginResponse = self.request(Method::Post, "login", params);
        self.session = resp.session;
        resp.handle
    }

    pub fn popup(&mut self, msg: &str) {
        // TODO: don't clone when we can properly do a partial
        // borrow of this struct
        let session = self.session.clone();
        let code = self.code.clone();
        let params: Vec<(&str, &str)> = vec![("session", &session),
                          ("room_code", &code),
                                             ("text", msg)];
        self.request(Method::Post, "login", params)
    }

    pub fn set_handle(&mut self, handle: &str) {
        // TODO: don't clone when we can properly do a partial
        // borrow of this struct
        let session = self.session.clone();
        let params: Vec<(&str, &str)> = vec![("session", &session),
                                             ("handle", handle)];
        self.request(Method::Put, "profile", params)
    }

    pub fn enqueue_song(&mut self, id: &str) -> String {
        // TODO: don't clone when we can properly do a partial
        // borrow of this struct
        let session = self.session.clone();
        let code = self.code.clone();
        let params: Vec<(&str, &str)> = vec![("session", &session),
                                             ("room_code", &code),
                                             ("song_id", id)];
        let resp: QueueResponse = self.request(Method::Post, "queue", params);
        resp.play_id
    }

    pub fn delete_song(&mut self, id: &str) {
        // TODO: don't clone when we can properly do a partial
        // borrow of this struct
        let session = self.session.clone();
        let code = self.code.clone();
        let params: Vec<(&str, &str)> = vec![("session", &session),
                                             ("room_code", &code),
                                             ("from", id)];
        self.request(Method::Delete, "queue", params)
    }

    pub fn favorites(&mut self) -> Vec<Song> {
        // TODO: don't clone when we can properly do a partial
        // borrow of this struct
        let session = self.session.clone();
        let params: Vec<(&str, &str)> = vec![("session", &session)];
        let resp: FavoritesResponse = self.request(Method::Get, "songs/favorites", params);
        let mut result = resp.songs;
        if resp.total_pages > 1 {
            result.reserve_exact(resp.total_entries as usize - resp.per_page as usize);
        }
        let mut num_pages = resp.total_pages;
        let mut cur_page = 1;
        while cur_page < num_pages {
            cur_page += 1;
            let page_as_str = format!("{}", cur_page);
            let params: Vec<(&str, &str)> = vec![("session", &session),
                                                 ("page", &page_as_str)];
            let mut resp: FavoritesResponse = self.request(Method::Get, "songs/favorites", params);
            result.append(&mut resp.songs);

            // Just in case we got more entries and an extra page
            num_pages = resp.total_pages;
        };
        result
    }
}
