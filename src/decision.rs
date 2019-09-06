//! This module contains the implementation of the Webmachine decision graph
//! which is available
//! [here](https://raw.githubusercontent.com/wiki/Webmachine/webmachine/images/http-headers-status-v3.png)

use futures::Future;
use hyper::{Body, Method, Request, Response, StatusCode};
use hyper::header::*;
use itertools::Itertools;
use mime::Mime;

use crate::resource::{PostResponse, Webmachine};
use crate::types::{
    RequestState,
    is_response_empty,
    request_time,
    set_response_header,
    trace
};

header! { (AirshipTrace, "Airship-Trace") => [String] }
header! { (AirshipQuip, "Airship-Quip") => [String] }

type BoxedFuture = Box<Future<Item = Response, Error = hyper::Error>>;

pub fn traverse<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    b13(r, req, state)
}

fn halt(status_code: StatusCode) -> BoxedFuture {
    Box::new(futures::future::ok(
        Response::new().with_status(status_code)
    ))
}

fn halt_with_response(status_code: StatusCode, state: &mut RequestState) -> BoxedFuture {
    let trace = state.decision_trace.join(",");
    let quip = String::from("blame me if inappropriate");

    let response = Response::new()
        .with_status(status_code)
        .with_header(Server::new("hyper/0.11.27"))
        .with_header(AirshipTrace(trace))
        .with_header(AirshipQuip(quip));
    Box::new(futures::future::ok(
        response
    ))
}

fn halt_with_header<H: Header>(status_code: StatusCode, hdr: H) -> BoxedFuture {
    Box::new(futures::future::ok(
        Response::new()
            .with_status(status_code)
            .with_header(hdr)
    ))
}


///////////////////////////////////////////////////////////////////////////////
// B column
///////////////////////////////////////////////////////////////////////////////

fn b13<R: Webmachine>(r: &R, _req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "b13");
    if r.service_available() {
        b12(r, _req, state)
    } else {
        halt(StatusCode::ServiceUnavailable)
    }
}

fn b12<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "b12");
    // known method
    let request_method = req.method();
    let known_methods = vec![Method::Get,
                             Method::Post,
                             Method::Head,
                             Method::Put,
                             Method::Delete,
                             Method::Trace,
                             Method::Connect,
                             Method::Options,
                             Method::Patch];
    let mut iter = known_methods.iter();
    match iter.find(|&m| m == request_method) {
        None    => halt(StatusCode::NotImplemented),
        Some(_) => b11(r, req, state)
    }
}

fn b11<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "b11");
    match r.uri_too_long(req.uri()) {
        true => halt(StatusCode::UriTooLong),
        false => b10(r, req, state)
    }
}

fn b10<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "b10");
    let request_method = req.method();
    let allowed_methods = r.allowed_methods();
    match allowed_methods.iter().find(|&m| m == request_method) {
        None => halt_with_header(StatusCode::MethodNotAllowed, Allow(allowed_methods)),
        Some(_) => b09(r, req, state)
    }
}

fn b09<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "b09");
    match r.malformed_request(req) {
        true => halt(StatusCode::BadRequest),
        false => b08(r, req, state)
    }
}

fn b08<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "b08");
    match r.is_authorized(req) {
        true => b07(r, req, state),
        false => halt(StatusCode::Unauthorized)
    }
}

fn b07<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "b07");
    match r.forbidden(req) {
        true => halt(StatusCode::Forbidden),
        false => b06(r, req, state)
    }
}

fn b06<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "b06");
    match r.valid_content_headers(req) {
        true => b05(r, req, state),
        false => halt(StatusCode::NotImplemented)
    }
}

fn b05<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "b05");
    match r.known_content_type(req) {
        true => b04(r, req, state),
        false => halt(StatusCode::UnsupportedMediaType)
    }
}

fn b04<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "b04");
    match r.entity_too_large(req) {
        true => halt(StatusCode::PayloadTooLarge),
        false => b03(r, req, state)
    }
}

fn b03<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "b03");
    match req.method() {
        Method::Options => {
            let allowed_methods = r.allowed_methods();
            halt_with_header(StatusCode::NoContent, Allow(allowed_methods))
        },
        _ =>
            c03(r, req, state)
    }
}

// ------------------------------------------------------------------------------
// -- C column
// ------------------------------------------------------------------------------

fn c04<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState, accept_header: &Accept) -> BoxedFuture {
    trace(state, "c04");
    let provided = r.content_types_provided();
    let result = map_accept_media(provided, &accept_header);
    match result {
        Some(_) => {
            state.matched_content_type = result;
            d04(r, req, state)
        },
        None => halt(StatusCode::NotAcceptable)
    }
}

fn c03<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "c03");
    match req.headers().get::<Accept>() {
        Some(ahdr) => c04(r, req, state, ahdr),
        None => d04(r, req, state)
    }
}

// ------------------------------------------------------------------------------
// -- D column
// ------------------------------------------------------------------------------

fn d05<R: Webmachine, H: Header>(r: &R, req: &Request, state: &mut RequestState, accept_lang_header: &H) -> BoxedFuture {
    trace(state, "d05");
    if r.language_available(accept_lang_header) {
        e05(r, req, state)
    } else {
        halt(StatusCode::NotAcceptable)
    }
}

fn d04<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "d04");
    match req.headers().get::<AcceptLanguage>() {
        Some(alhdr) => d05(r, req, state, alhdr),
        None        => e05(r, req, state)
    }
}

// ------------------------------------------------------------------------------
// -- E column
// ------------------------------------------------------------------------------

fn e06<R: Webmachine, H: Header>(r: &R, req: &Request, state: &mut RequestState, _accept_charset_header: &H) -> BoxedFuture {
    trace(state, "e06");
    //TODO: Implement charset negotiation
    f06(r, req, state)
}

fn e05<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "e05");
    match req.headers().get::<AcceptCharset>() {
        Some(achdr) => e06(r, req, state, achdr),
        None => f06(r, req, state)
    }
}

// ------------------------------------------------------------------------------
// -- F column
// ------------------------------------------------------------------------------

fn f07<R: Webmachine, H: Header>(r: &R, req: &Request, state: &mut RequestState, _accept_encoding_header: &H) -> BoxedFuture {
    trace(state, "f07");
    //TODO: Implement encoding negotiation
    f06(r, req, state)
}

fn f06<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "f06");
    match req.headers().get::<AcceptEncoding>() {
        Some(aehdr) => f07(r, req, state, aehdr),
        None => g07(r, req, state)
    }
}

// ------------------------------------------------------------------------------
// -- G column
// ------------------------------------------------------------------------------

fn g11<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState, etags: &Vec<EntityTag>) -> BoxedFuture {
    trace(state, "g11");
    match etags.is_empty() {
        true => halt(StatusCode::PreconditionFailed),
        false => h10(r, req, state)
    }
}

fn g09<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState, if_match: &IfMatch) -> BoxedFuture {
    trace(state, "g09");
    match if_match {
        IfMatch::Any => h10(r, req, state),
        IfMatch::Items(etags) => g11(r, req, state, etags)
    }
}

fn g08<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "g08");
    match req.headers().get::<IfMatch>() {
        Some(imhdr) => g09(r, req, state, &imhdr),
        None => h10(r, req, state)
    }
}

fn g07<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "g07");
    // TODO: set Vary headers
    match r.resource_exists() {
        true  => g08(r, req, state),
        false => h07(r, req, state)
    }
}

// ------------------------------------------------------------------------------
// -- H column
// ------------------------------------------------------------------------------

fn h12<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "h12");
    let m_if_unmod_since = req.headers().get::<IfUnmodifiedSince>();
    let m_last_modified = r.last_modified();
    match (m_if_unmod_since, m_last_modified) {
        (Some(if_unmod_since), Some(last_modified))
            if last_modified > **if_unmod_since => halt(StatusCode::PreconditionFailed),
        _                                       => i12(r, req, state)
    }
}

fn h11<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "h11");
    // TODO: Revisit this to understand how hyper handles invalid HttpDate values
    // let m_header_str = req.headers().get_raw("if-unmodified-since");
    // let valid_date = match m_header_str {
    //     Some(header_raw) => Header::parse_header(header_raw).is_ok(),
    //     None             => &false
    // };
    let valid_date = true;
    match valid_date {
        true  => h12(r, req, state),
        false => i12(r, req, state)
    }
}

fn h10<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "h10");
    match req.headers().has::<IfUnmodifiedSince>() {
        true => h11(r, req, state),
        false => i12(r, req, state)
    }
}

fn h07<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "h07");
    match req.headers().get::<IfMatch>() {
        Some(IfMatch::Any) => halt(StatusCode::PreconditionFailed),
        _                  => i07(r, req, state)
    }
}

// ------------------------------------------------------------------------------
// -- I column
// ------------------------------------------------------------------------------

fn i13<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState, if_none_match: &IfNoneMatch) -> BoxedFuture {
    trace(state, "i13");
    match if_none_match {
        IfNoneMatch::Any          => j18(r, req, state),
        IfNoneMatch::Items(etags) => k13(r, req, state, &etags)
    }
}

fn i12<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "i12");
    match req.headers().get::<IfNoneMatch>() {
        Some(inmhdr) => i13(r, req, state, inmhdr),
        None => l13(r, req, state)
    }
}


fn i07<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "i07");
    match req.method() {
        Method::Put => i04(r, req, state),
        _           => k07(r, req, state)
    }
}

fn i04<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "i04");
    match r.moved_permanently() {
        Some(location) => {
            set_response_header(state, Location::new(location));
            halt(StatusCode::MovedPermanently)
        },
        None => p03(r, req, state)
    }
}

// ------------------------------------------------------------------------------
// -- J column
// ------------------------------------------------------------------------------

fn j18<R: Webmachine>(_r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "j18");
    match req.method() {
        Method::Get  => halt(StatusCode::NotModified),
        Method::Head => halt(StatusCode::NotModified),
        _            => halt(StatusCode::PreconditionFailed)
    }
}

// ------------------------------------------------------------------------------
// -- K column
// ------------------------------------------------------------------------------

fn k13<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState, etags: &Vec<EntityTag>) -> BoxedFuture {
    trace(state, "k13");
    match etags.is_empty() {
        true  => l13(r, req, state),
        false => j18(r, req, state)
    }
}

fn k07<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "k07");
    match r.previously_existed() {
        true  => k05(r, req, state),
        false => l07(r, req, state)
    }
}

fn k05<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "k05");
    match r.moved_permanently() {
        Some(location) => {
            set_response_header(state, Location::new(location));
            halt(StatusCode::MovedPermanently)
        },
        None           => l05(r, req, state)
    }
}

// ------------------------------------------------------------------------------
// -- L column
// ------------------------------------------------------------------------------

fn l17<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "l17");
    let m_if_mod_since = req.headers().get::<IfModifiedSince>();
    let m_last_modified = r.last_modified();
    match (m_if_mod_since, m_last_modified) {
        (Some(if_mod_since), Some(last_modified))
            if **if_mod_since > last_modified => m16(r, req, state),
        _                                     => halt(StatusCode::NotModified)
    }
}

fn l15<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "l15");
    let m_if_mod_since = req.headers().get::<IfModifiedSince>();
    match m_if_mod_since {
        Some(if_mod_since)
            if **if_mod_since > request_time(state) => m16(r, req, state),
        _                                           => l17(r, req, state)
    }
}

fn l14<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "l14");
    // TODO: Revisit this to understand how hyper handles invalid HttpDate values
    // let valid_date = req.headers()
    //     .get_raw("if-modified-since")
    //     .and_then(str::parse::<HttpDate>())
    //     .is_some();
    let valid_date = true;
    match valid_date {
        true  => l15(r, req, state),
        false => m16(r, req, state)
    }
}

fn l13<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "l13");
    match req.headers().has::<IfModifiedSince>() {
        true  => l14(r, req, state),
        false => m16(r, req, state)
    }
}

fn l07<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "l07");
    match req.method() {
        Method::Post => m07(r, req, state),
        _            => halt(StatusCode::NotFound)
    }
}

fn l05<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "l05");
    match r.moved_temporarily() {
        Some(location) => {
            set_response_header(state, Location::new(location));
            halt(StatusCode::TemporaryRedirect)
        },
        None           => m05(r, req, state)
    }
}

// ------------------------------------------------------------------------------
// -- M column
// ------------------------------------------------------------------------------

fn m20<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "m20");
    match (r.delete_resource(req), r.delete_completed()) {
        (true, true)  => o20(r, req, state),
        (true, false) => halt(StatusCode::Accepted),
        _             => halt(StatusCode::InternalServerError)
    }
}

fn m16<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "m16");
    match req.method() {
        Method::Delete => m20(r, req, state),
        _              => n16(r, req, state)
    }
}


fn m07<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "m07");
    match r.allow_missing_post() {
        true  => n11(r, req, state),
        false => halt(StatusCode::NotFound)
    }
}


fn m05<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "m05");
    match req.method() {
        Method::Post => n05(r, req, state),
        _            => halt(StatusCode::Gone)
    }
}

// ------------------------------------------------------------------------------
// -- N column
// ------------------------------------------------------------------------------

fn n16<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "n16");
    match req.method() {
        Method::Post => n11(r, req, state),
        _            => o16(r, req, state)
    }
}

fn n11<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "n11");
    let post_response = r.process_post(req);
    process_post_action(r, req, state, post_response)
}

fn n05<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "n05");
    match r.allow_missing_post() {
        true  => n11(r, req, state),
        false => halt(StatusCode::Gone)
    }
}

// ------------------------------------------------------------------------------
// -- O column
// ------------------------------------------------------------------------------

fn o20<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "o20");
    match is_response_empty(state) {
        true  => halt(StatusCode::Created),
        false => o18(r, req, state)
    }
}


fn o18<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "o18");
    if r.multiple_choices() {
        halt(StatusCode::MultipleChoices)
    } else {
        match req.method() {
            // TODO: set expiration, etc. headers
            Method::Get | Method::Head  => {
                let (content_type, body_fn) =
                    state.matched_content_type.take().unwrap_or_else(|| {
                        // TODO: This unwrap should be safe because if we've
                        // made it this far in the decision processing then we
                        // know there is at least one entry in the
                        // content_types_provided vector, but I want to confirm
                        // this is absolutlely the case.
                        r.content_types_provided().first().unwrap().clone()
                    });
                set_response_header(state, ContentType(Mime::clone(&content_type)));
                let response_body = body_fn(req);
                state.response.set_body(response_body);
            },
            _  => ()
        };
        if let Some(etag) = r.generate_etag(req) {
            set_response_header(state, ETag(etag));
        }
        if let Some(modified) = r.last_modified() {
            set_response_header(state, LastModified(modified));
        }
        halt_with_response(StatusCode::Ok, state)
    }
}

fn o16<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "o16");
    match req.method() {
        Method::Put => o14(r, req, state),
        _           => o17(r, req, state)
    }
}

fn o17<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "o17");
    match req.method() {
        Method::Patch => {
            let accepted = r.patch_content_types_accepted();
            let result = req.headers().get::<ContentType>()
                .and_then(|ct_hdr| {
                    map_content_media::<()>(accepted, ct_hdr)
                });
            match result {
                Some(action) => {
                    action(req);
                    o20(r, req, state)
                },
                None => halt(StatusCode::UnsupportedMediaType)
            }
        },
        _ => o18(r, req, state)
    }
}

fn o14<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "o14");
    if r.is_conflict() {
        halt(StatusCode::Conflict)
    } else {
        let accepted = r.content_types_accepted();
        let result = req.headers().get::<ContentType>()
            .and_then(|ct_hdr| {
                map_content_media::<()>(accepted, ct_hdr)
            });
        match result {
            Some(action) => {
                action(req);
                p11(r, req, state)
            },
            None => halt(StatusCode::UnsupportedMediaType)
        }
    }
}

// ------------------------------------------------------------------------------
// -- P column
// ------------------------------------------------------------------------------

fn p11<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "p11");
    match req.headers().has::<Location>() {
        true => halt(StatusCode::Created),
        false => o20(r, req, state)
    }
}

fn p03<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "p03");
    if r.is_conflict() {
        halt(StatusCode::Conflict)
    } else {
        let accepted = r.content_types_accepted();
        let result = req.headers().get::<ContentType>()
            .and_then(|ct_hdr| {
                map_content_media::<()>(accepted, ct_hdr)
            });
        match result {
            Some(action) => {
                action(req);
                p11(r, req, state)
            },
            None => halt(StatusCode::UnsupportedMediaType)
        }
    }
}

// ------------------------------------------------------------------------------
// -- Decision helper functions
// ------------------------------------------------------------------------------

/// Matches a list of server-side parsing options against a the client-side
/// content value.
fn map_content_media<T>(
    provided: Vec<(Mime, fn(&Request) -> T)>,
    content_type: &ContentType
) -> Option<fn(&Request) -> T> {
    let mut action_match = None;

    // Iterate through all of the provided Content-Types for the
    // resource and look for a match.
    for (ct_hdr, action) in &provided {
        if ct_hdr == &content_type.0 {
            action_match = Some(action.clone());
            break;
        }
    }
    action_match
}

/// Matches a list of server-side resource options against a quality-marked list
/// of client-side preferences.
fn map_accept_media(
    provided: Vec<(Mime, fn(&Request) -> Body)>,
    accept: &Accept
) -> Option<(Mime, fn(&Request) -> Body)> {
    let zero_quality = q(0);
    let mut match_quality = q(0);
    let mut type_match = None;

    for a_hdr in accept.iter() {
        if a_hdr.quality == zero_quality {
            // Do not match Accept header values with a quality of zero
            break;
        } else {
            // Iterate through all of the provided Content-Types for the
            // resource and find the match with the highest quality value.
            for (ct_hdr, body_fn) in &provided {
                if a_hdr.item == mime::STAR_STAR && a_hdr.quality > match_quality {
                    type_match = Some((ct_hdr.clone(), body_fn.clone()));
                    match_quality = a_hdr.quality;
                } else if a_hdr.item.type_() == ct_hdr.type_() && a_hdr.quality > match_quality {
                    if a_hdr.item.subtype() == ct_hdr.subtype() || a_hdr.item.subtype() == mime::STAR {
                        type_match = Some((ct_hdr.clone(), body_fn.clone()));
                        match_quality = a_hdr.quality;
                    }
                }
            }
        }
    }
    type_match
}

fn append_request_path(req: &Request, path_segments: &Vec<String>) -> String {
    let path_suffix: String =
        path_segments
        .iter()
        .cloned()
        .intersperse(",".to_string())
        .collect();
    [req.path(), &path_suffix].concat().into()
}

fn create<R: Webmachine>(
    r: &R,
    req: &Request,
    state: &mut RequestState,
    path_segments: &Vec<String>
) -> Option<()> {
    let location = append_request_path(req, path_segments);
    set_response_header(state, Location::new(location));
    let accepted = r.content_types_accepted();
    // negotiate_content_types_accepted::<()>(&accepted, req);
    req.headers().get::<ContentType>()
        .and_then(|ct_hdr| {
            map_content_media::<()>(accepted, ct_hdr)
        })
        .and_then(|action| {
            action(req);
            Some(())
        })
}

fn process_post_action<R: Webmachine>(
    r: &R,
    req: &Request,
    state: &mut RequestState,
    pr: PostResponse
) -> BoxedFuture
{
    match pr {
        PostResponse::PostCreate(ref path_segments) => {
            match create(r, req, state, path_segments) {
                Some(()) => p11(r, req, state),
                None => halt(StatusCode::UnsupportedMediaType)
            }
        },
        PostResponse::PostCreateRedirect(ref path_segments) => {
            match create(r, req, state, path_segments) {
                Some(()) => halt(StatusCode::SeeOther),
                None => halt(StatusCode::UnsupportedMediaType)
            }
        },
        PostResponse::PostProcess(accepted) => {
            let result = req.headers().get::<ContentType>()
                .and_then(|ct_hdr| {
                    map_content_media::<()>(accepted, ct_hdr)
                });
            match result {
                Some(action) => {
                    action(req);
                    p11(r, req, state)
                },
                None => halt(StatusCode::UnsupportedMediaType)
            }
        },
        PostResponse::PostProcessRedirect(accepted) => {
            let result = req.headers().get::<ContentType>()
                .and_then(|ct_hdr| {
                    map_content_media::<String>(accepted, ct_hdr)
                });
            match result {
                Some(action) => {
                    let location = action(req);
                    set_response_header(state, Location::new(location));
                    halt(StatusCode::SeeOther)
                },
                None => halt(StatusCode::UnsupportedMediaType)
            }
        }
    }
}
