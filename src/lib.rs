extern crate hyper;
extern crate rustc_serialize;

use hyper::Client;
use hyper::header::Connection;
use hyper::Url;
use std::io::Read;
use rustc_serialize::json::{self, Decoder};

#[derive(RustcDecodable)]
struct LoginResponse {
    session: String,
    email: String,
    handle: String,
    color: String,
    hide_handle_in_queue: bool
}

#[derive(RustcDecodable)]
struct QueueResponse {
    index: u32,
    song_id: u32,
    play_id: String,
    title: String,
    artist: String,
    duration: u32
}

const ROOM_CODE: &'static str = "DNSC";
const EMAIL: &'static str = "me@branan.info";

struct Voicebox {
    code: String,
    client: Client,
    session: String,
}

fn build_url(endpoint: &str, params: Vec<(&str, &str)>) -> String {
    let mut url = Url::parse(&format!("http://voiceboxpdx.com/api/v1/{}.json", endpoint)).unwrap();
    url.set_query_from_pairs(params.into_iter());
    url.serialize()
}

impl Voicebox {
    fn new(room_code: &str) -> Voicebox {
        Voicebox { code: room_code.to_owned(), client: Client::new(), session: String::new() }
    }

    fn login(&mut self, email: &str) -> String {
        let params = vec![("email", email)];
        let url = build_url("login", params);
        let mut res = self.client.post(&url).send().unwrap();
        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();
        let resp: LoginResponse = json::decode(&body).unwrap();
        self.session = resp.session;
        resp.handle
    }

    fn popup(&mut self, msg: &str) {
        let params: Vec<(&str, &str)> = vec![("session", &self.session),
                          ("room_code", &self.code),
                          ("text", msg)];
        let url = build_url("popups", params);
        let mut res = self.client.post(&url).send().unwrap();
        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();
    }

    fn set_handle(&mut self, handle: &str) {
        let params: Vec<(&str, &str)> = vec![("session", &self.session),
                          ("handle", handle)];
        let url = build_url("profile", params);
        let mut res = self.client.put(&url).send().unwrap();
        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();
    }

    fn enqueue_song(&mut self, id: &str) -> String {
        let params: Vec<(&str, &str)> = vec![("session", &self.session),
                          ("room_code", &self.code),
                          ("song_id", id)];
        let url = build_url("queue", params);
        let mut res = self.client.post(&url).send().unwrap();
        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();
        let resp: QueueResponse = json::decode(&body).unwrap();
        resp.play_id
    }

    fn delete_song(&mut self, id: &str) {
        let params: Vec<(&str, &str)> = vec![("session", &self.session),
                          ("room_code", &self.code),
                          ("from", id)];
        let url = build_url("queue", params);
        let mut res = self.client.delete(&url).send().unwrap();
        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();
    }
}
