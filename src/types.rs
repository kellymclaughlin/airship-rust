use std::time::SystemTime;
use std::rc::Rc;

use hyper::{Body, Request, Response};
use hyper::header::*;

use mime::Mime;

// type ErrorResponses = Map HTTP.Status [(MediaType, ResponseBody)]
pub type ErrorResponses = String;
// pub type ResponseBody = <T: Into<B>>;


pub struct RequestState {
    pub error_responses: ErrorResponses,
    pub decision_trace: Vec<String>,
    pub matched_content_type: Rc<Option<(Mime, Box<Fn(&Request) -> Body>)>>,
    pub response: Box<Response>,
    pub request_time: SystemTime
}

impl RequestState {
    pub fn new() -> RequestState {
        RequestState {
            error_responses: String::from(""),
            decision_trace: vec![],
            matched_content_type: Rc::new(None),
            response: Box::new(Response::new()),
            request_time: SystemTime::now()
        }
    }
}

pub fn trace(state: &mut RequestState, t: &str) -> () {
    state.decision_trace.push(String::from(t));
    ()
}

pub fn set_matched_content_type(state: &mut RequestState, matched: Rc<Option<(Mime, Box<Fn(&Request) -> Body>)>>) -> () {
    state.matched_content_type = matched;
    ()
}

pub fn set_response_header<H: Header>(state: &mut RequestState, hdr: H) -> () {
    let response = &mut state.response;
    response.headers_mut().set(hdr);
    ()
}

pub fn request_time(state: &RequestState) -> HttpDate {
    HttpDate::from(state.request_time)
}

pub fn is_response_empty(state: &mut RequestState) -> bool {
    let response = &mut state.response;
    match response.body_ref() {
        Some(b) => b.is_empty(),
        None => false
    }
}
