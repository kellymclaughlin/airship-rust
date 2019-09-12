use std::time::SystemTime;

use hyper::{Body, Request, Response};
use hyper::header::*;

use mime::Mime;

// type ErrorResponses = Map HTTP.Status [(MediaType, ResponseBody)]
pub type ErrorResponses = String;

pub struct AirshipState {
    pub error_responses: ErrorResponses,
    pub decision_trace: Vec<String>,
    pub matched_content_type: Option<(Mime, fn(&Request) -> Body)>,
    pub response: Box<Response>,
    pub request_time: SystemTime
}

impl AirshipState {
    pub fn new() -> AirshipState {
        AirshipState {
            error_responses: String::from(""),
            decision_trace: vec![],
            matched_content_type: None,
            response: Box::new(Response::new()),
            request_time: SystemTime::now()
        }
    }
}

pub trait HasAirshipState {
    fn get_airship_state_mut(&mut self) -> &mut AirshipState;
    fn get_airship_state(&self) -> &AirshipState;
}

pub fn get_trace<S>(state: &S) -> &Vec<String>
where
    S: HasAirshipState
{
    let airship_state = state.get_airship_state();
    &airship_state.decision_trace
}

pub fn trace<S>(state: &mut S, t: &str)
where
    S: HasAirshipState
{
    let airship_state = state.get_airship_state_mut();
    airship_state.decision_trace.push(String::from(t));
}

pub fn get_matched_content_type<S>(
    state: &mut S
) -> &mut Option<(Mime, fn(&Request) -> Body)>
where
    S: HasAirshipState
{
    let airship_state = state.get_airship_state_mut();
    &mut airship_state.matched_content_type
}

pub fn matched_content_type<S>(
    state: &mut S,
    matched: Option<(Mime, fn(&Request) -> Body)>
)
where
    S: HasAirshipState
{
    let airship_state = state.get_airship_state_mut();
    airship_state.matched_content_type = matched;
}

pub fn set_response_header<H, S>(
    state: &mut S,
    hdr: H
)
where
    H: Header,
    S: HasAirshipState
{
    let airship_state = state.get_airship_state_mut();
    let response = &mut airship_state.response;
    response.headers_mut().set(hdr);
}

pub fn request_time<S>(state: &S) -> HttpDate
where
    S: HasAirshipState
{
    let airship_state = state.get_airship_state();
    HttpDate::from(airship_state.request_time)
}

pub fn is_response_empty<S>(state: &S) -> bool
where
    S: HasAirshipState
{
    let airship_state = state.get_airship_state();
    let response = &airship_state.response;
    match response.body_ref() {
        Some(b) => b.is_empty(),
        None => false
    }
}

pub fn get_response<S>(
    state: &mut S
) -> &mut Box<Response>
where
    S: HasAirshipState
{
    let airship_state = state.get_airship_state_mut();
    &mut airship_state.response
}


pub struct RequestState(AirshipState);

impl RequestState {
    pub fn new() -> RequestState {
        RequestState(AirshipState::new())
    }
}

impl HasAirshipState for RequestState {
    fn get_airship_state(&self) -> &AirshipState {
        &self.0
    }

    fn get_airship_state_mut(&mut self) -> &mut AirshipState {
        &mut self.0
    }
}
