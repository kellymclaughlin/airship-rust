//! This module contains the implementation of the Webmachine decision graph
//! which is available
//! [here](https://raw.githubusercontent.com/wiki/Webmachine/webmachine/images/http-headers-status-v3.png)

#![allow(clippy::type_complexity)]

use futures::Future;
use hyper::header::*;
use hyper::{Body, Method, Request, Response, StatusCode};
use itertools::Itertools;
use mime::Mime;

use crate::resource::{PostResponse, Webmachine};
use crate::types::*;

header! { (AirshipTrace, "Airship-Trace") => [String] }
header! { (AirshipQuip, "Airship-Quip") => [String] }

type BoxedFuture = Box<dyn Future<Item = Response, Error = hyper::Error>>;

pub fn traverse<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    b13(r, req, state)
}

fn halt<S: HasAirshipState>(
    status_code: StatusCode,
    state: &mut S,
) -> BoxedFuture {
    let trace = get_trace(state).join(",");
    let quip = String::from("blame me if inappropriate");

    Box::new(futures::future::ok(
        Response::new()
            .with_status(status_code)
            .with_header(Server::new("hyper/0.11.27"))
            .with_header(AirshipTrace(trace))
            .with_header(AirshipQuip(quip)),
    ))
}

fn halt_with_response<S: HasAirshipState>(
    status_code: StatusCode,
    state: &mut S,
) -> BoxedFuture {
    let trace = get_trace(state).join(",");
    let quip = String::from("blame me if inappropriate");

    let response = get_response(state)
        .with_status(status_code)
        .with_header(Server::new("hyper/0.11.27"))
        .with_header(AirshipTrace(trace))
        .with_header(AirshipQuip(quip));

    Box::new(futures::future::ok(response))
}

fn halt_with_header<H: Header, S: HasAirshipState>(
    status_code: StatusCode,
    hdr: H,
    state: &mut S,
) -> BoxedFuture {
    let trace = get_trace(state).join(",");
    let quip = String::from("blame me if inappropriate");

    Box::new(futures::future::ok(
        Response::new()
            .with_status(status_code)
            .with_header(hdr)
            .with_header(Server::new("hyper/0.11.27"))
            .with_header(AirshipTrace(trace))
            .with_header(AirshipQuip(quip)),
    ))
}

///////////////////////////////////////////////////////////////////////////////
// B column
///////////////////////////////////////////////////////////////////////////////

fn b13<R, S>(r: &R, _req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "b13");
    if r.service_available(state) {
        b12(r, _req, state)
    } else {
        halt(StatusCode::ServiceUnavailable, state)
    }
}

fn b12<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "b12");
    // known method
    let request_method = req.method();
    let known_methods = vec![
        Method::Get,
        Method::Post,
        Method::Head,
        Method::Put,
        Method::Delete,
        Method::Trace,
        Method::Connect,
        Method::Options,
        Method::Patch,
    ];
    let mut iter = known_methods.iter();
    match iter.find(|&m| m == request_method) {
        None => halt(StatusCode::NotImplemented, state),
        Some(_) => b11(r, req, state),
    }
}

fn b11<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "b11");
    if r.uri_too_long(state, req.uri()) {
        halt(StatusCode::UriTooLong, state)
    } else {
        b10(r, req, state)
    }
}

fn b10<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "b10");
    let request_method = req.method();
    let allowed_methods = r.allowed_methods(state);
    match allowed_methods.iter().find(|&m| m == request_method) {
        None => halt_with_header(
            StatusCode::MethodNotAllowed,
            Allow(allowed_methods),
            state,
        ),
        Some(_) => b09(r, req, state),
    }
}

fn b09<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "b09");
    if r.malformed_request(state, req) {
        halt(StatusCode::BadRequest, state)
    } else {
        b08(r, req, state)
    }
}

fn b08<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "b08");
    if r.is_authorized(state, req) {
        b07(r, req, state)
    } else {
        halt(StatusCode::Unauthorized, state)
    }
}

fn b07<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "b07");
    if r.forbidden(state, req) {
        halt(StatusCode::Forbidden, state)
    } else {
        b06(r, req, state)
    }
}

fn b06<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "b06");
    if r.valid_content_headers(state, req) {
        b05(r, req, state)
    } else {
        halt(StatusCode::NotImplemented, state)
    }
}

fn b05<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "b05");
    if r.known_content_type(state, req) {
        b04(r, req, state)
    } else {
        halt(StatusCode::UnsupportedMediaType, state)
    }
}

fn b04<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "b04");
    if r.entity_too_large(state, req) {
        halt(StatusCode::PayloadTooLarge, state)
    } else {
        b03(r, req, state)
    }
}

fn b03<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "b03");
    match req.method() {
        Method::Options => {
            let allowed_methods = r.allowed_methods(state);
            halt_with_header(
                StatusCode::NoContent,
                Allow(allowed_methods),
                state,
            )
        }
        _ => c03(r, req, state),
    }
}

// ------------------------------------------------------------------------------
// -- C column
// ------------------------------------------------------------------------------

fn c04<R, S>(
    r: &R,
    req: &Request,
    state: &mut S,
    accept_header: &Accept,
) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "c04");
    let provided = r.content_types_provided(state);
    let result = map_accept_media(provided, &accept_header);
    match result {
        Some(_) => {
            matched_content_type(state, result);
            d04(r, req, state)
        }
        None => halt(StatusCode::NotAcceptable, state),
    }
}

fn c03<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "c03");
    match req.headers().get::<Accept>() {
        Some(ahdr) => c04(r, req, state, ahdr),
        None => d04(r, req, state),
    }
}

// ------------------------------------------------------------------------------
// -- D column
// ------------------------------------------------------------------------------

fn d05<R, H, S>(
    r: &R,
    req: &Request,
    state: &mut S,
    accept_lang_header: &H,
) -> BoxedFuture
where
    H: Header,
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "d05");
    if r.language_available(state, accept_lang_header) {
        e05(r, req, state)
    } else {
        halt(StatusCode::NotAcceptable, state)
    }
}

fn d04<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "d04");
    match req.headers().get::<AcceptLanguage>() {
        Some(alhdr) => d05(r, req, state, alhdr),
        None => e05(r, req, state),
    }
}

// ------------------------------------------------------------------------------
// -- E column
// ------------------------------------------------------------------------------

fn e06<R, H, S>(
    r: &R,
    req: &Request,
    state: &mut S,
    _accept_charset_header: &H,
) -> BoxedFuture
where
    H: Header,
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "e06");
    //TODO: Implement charset negotiation
    f06(r, req, state)
}

fn e05<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "e05");
    match req.headers().get::<AcceptCharset>() {
        Some(achdr) => e06(r, req, state, achdr),
        None => f06(r, req, state),
    }
}

// ------------------------------------------------------------------------------
// -- F column
// ------------------------------------------------------------------------------

fn f07<R, H, S>(
    r: &R,
    req: &Request,
    state: &mut S,
    _accept_encoding_header: &H,
) -> BoxedFuture
where
    H: Header,
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "f07");
    //TODO: Implement encoding negotiation
    f06(r, req, state)
}

fn f06<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "f06");
    match req.headers().get::<AcceptEncoding>() {
        Some(aehdr) => f07(r, req, state, aehdr),
        None => g07(r, req, state),
    }
}

// ------------------------------------------------------------------------------
// -- G column
// ------------------------------------------------------------------------------

fn g11<R, S>(
    r: &R,
    req: &Request,
    state: &mut S,
    etags: &[EntityTag],
) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "g11");
    if etags.is_empty() {
        halt(StatusCode::PreconditionFailed, state)
    } else {
        h10(r, req, state)
    }
}

fn g09<R, S>(
    r: &R,
    req: &Request,
    state: &mut S,
    if_match: &IfMatch,
) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "g09");
    match if_match {
        IfMatch::Any => h10(r, req, state),
        IfMatch::Items(etags) => g11(r, req, state, etags),
    }
}

fn g08<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "g08");
    match req.headers().get::<IfMatch>() {
        Some(imhdr) => g09(r, req, state, &imhdr),
        None => h10(r, req, state),
    }
}

fn g07<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "g07");
    // TODO: set Vary headers
    if r.resource_exists(state) {
        g08(r, req, state)
    } else {
        h07(r, req, state)
    }
}

// ------------------------------------------------------------------------------
// -- H column
// ------------------------------------------------------------------------------

fn h12<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "h12");
    let m_if_unmod_since = req.headers().get::<IfUnmodifiedSince>();
    let m_last_modified = r.last_modified(state);
    match (m_if_unmod_since, m_last_modified) {
        (Some(if_unmod_since), Some(last_modified))
            if last_modified > **if_unmod_since =>
        {
            halt(StatusCode::PreconditionFailed, state)
        }
        _ => i12(r, req, state),
    }
}

fn h11<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "h11");
    // TODO: Revisit this to understand how hyper handles invalid HttpDate values
    // let m_header_str = req.headers().get_raw("if-unmodified-since");
    // let valid_date = match m_header_str {
    //     Some(header_raw) => Header::parse_header(header_raw).is_ok(),
    //     None             => &false
    // };
    let valid_date = true;
    if valid_date {
        h12(r, req, state)
    } else {
        i12(r, req, state)
    }
}

fn h10<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "h10");
    if req.headers().has::<IfUnmodifiedSince>() {
        h11(r, req, state)
    } else {
        i12(r, req, state)
    }
}

fn h07<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "h07");
    match req.headers().get::<IfMatch>() {
        Some(IfMatch::Any) => halt(StatusCode::PreconditionFailed, state),
        _ => i07(r, req, state),
    }
}

// ------------------------------------------------------------------------------
// -- I column
// ------------------------------------------------------------------------------

fn i13<R, S>(
    r: &R,
    req: &Request,
    state: &mut S,
    if_none_match: &IfNoneMatch,
) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "i13");
    match if_none_match {
        IfNoneMatch::Any => j18(r, req, state),
        IfNoneMatch::Items(etags) => k13(r, req, state, &etags),
    }
}

fn i12<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "i12");
    match req.headers().get::<IfNoneMatch>() {
        Some(inmhdr) => i13(r, req, state, inmhdr),
        None => l13(r, req, state),
    }
}

fn i07<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "i07");
    match req.method() {
        Method::Put => i04(r, req, state),
        _ => k07(r, req, state),
    }
}

fn i04<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "i04");
    match r.moved_permanently(state) {
        Some(location) => {
            set_response_header(state, Location::new(location));
            halt(StatusCode::MovedPermanently, state)
        }
        None => p03(r, req, state),
    }
}

// ------------------------------------------------------------------------------
// -- J column
// ------------------------------------------------------------------------------

fn j18<R, S>(_r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "j18");
    match req.method() {
        Method::Get => halt(StatusCode::NotModified, state),
        Method::Head => halt(StatusCode::NotModified, state),
        _ => halt(StatusCode::PreconditionFailed, state),
    }
}

// ------------------------------------------------------------------------------
// -- K column
// ------------------------------------------------------------------------------

fn k13<R, S>(
    r: &R,
    req: &Request,
    state: &mut S,
    etags: &[EntityTag],
) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "k13");
    if etags.is_empty() {
        l13(r, req, state)
    } else {
        j18(r, req, state)
    }
}

fn k07<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "k07");
    if r.previously_existed(state) {
        k05(r, req, state)
    } else {
        l07(r, req, state)
    }
}

fn k05<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "k05");
    match r.moved_permanently(state) {
        Some(location) => {
            set_response_header(state, Location::new(location));
            halt(StatusCode::MovedPermanently, state)
        }
        None => l05(r, req, state),
    }
}

// ------------------------------------------------------------------------------
// -- L column
// ------------------------------------------------------------------------------

fn l17<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "l17");
    let m_if_mod_since = req.headers().get::<IfModifiedSince>();
    let m_last_modified = r.last_modified(state);
    match (m_if_mod_since, m_last_modified) {
        (Some(if_mod_since), Some(last_modified))
            if **if_mod_since > last_modified =>
        {
            m16(r, req, state)
        }
        _ => halt(StatusCode::NotModified, state),
    }
}

fn l15<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "l15");
    let m_if_mod_since = req.headers().get::<IfModifiedSince>();
    match m_if_mod_since {
        Some(if_mod_since) if **if_mod_since > request_time(state) => {
            m16(r, req, state)
        }
        _ => l17(r, req, state),
    }
}

fn l14<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "l14");
    // TODO: Revisit this to understand how hyper handles invalid HttpDate values
    // let valid_date = req.headers()
    //     .get_raw("if-modified-since")
    //     .and_then(str::parse::<HttpDate>())
    //     .is_some();
    let valid_date = true;
    if valid_date {
        l15(r, req, state)
    } else {
        m16(r, req, state)
    }
}

fn l13<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "l13");
    if req.headers().has::<IfModifiedSince>() {
        l14(r, req, state)
    } else {
        m16(r, req, state)
    }
}

fn l07<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "l07");
    match req.method() {
        Method::Post => m07(r, req, state),
        _ => halt(StatusCode::NotFound, state),
    }
}

fn l05<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "l05");
    match r.moved_temporarily(state) {
        Some(location) => {
            set_response_header(state, Location::new(location));
            halt(StatusCode::TemporaryRedirect, state)
        }
        None => m05(r, req, state),
    }
}

// ------------------------------------------------------------------------------
// -- M column
// ------------------------------------------------------------------------------

fn m20<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "m20");
    match (r.delete_resource(state, req), r.delete_completed(state)) {
        (true, true) => o20(r, req, state),
        (true, false) => halt(StatusCode::Accepted, state),
        _ => halt(StatusCode::InternalServerError, state),
    }
}

fn m16<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "m16");
    match req.method() {
        Method::Delete => m20(r, req, state),
        _ => n16(r, req, state),
    }
}

fn m07<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "m07");
    if r.allow_missing_post(state) {
        n11(r, req, state)
    } else {
        halt(StatusCode::NotFound, state)
    }
}

fn m05<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "m05");
    match req.method() {
        Method::Post => n05(r, req, state),
        _ => halt(StatusCode::Gone, state),
    }
}

// ------------------------------------------------------------------------------
// -- N column
// ------------------------------------------------------------------------------

fn n16<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "n16");
    match req.method() {
        Method::Post => n11(r, req, state),
        _ => o16(r, req, state),
    }
}

fn n11<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "n11");
    let post_response = r.process_post(state, req);
    process_post_action(r, req, state, post_response)
}

fn n05<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "n05");
    if r.allow_missing_post(state) {
        n11(r, req, state)
    } else {
        halt(StatusCode::Gone, state)
    }
}

// ------------------------------------------------------------------------------
// -- O column
// ------------------------------------------------------------------------------

fn o20<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "o20");
    if is_response_empty(state) {
        halt(StatusCode::Created, state)
    } else {
        o18(r, req, state)
    }
}

fn o18<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "o18");
    if r.multiple_choices(state) {
        halt(StatusCode::MultipleChoices, state)
    } else {
        match req.method() {
            // TODO: set expiration, etc. headers
            Method::Get | Method::Head => {
                let (content_type, body_fn) = get_matched_content_type(state)
                    .take()
                    .unwrap_or_else(|| {
                        // TODO: This unwrap should be safe because if we've
                        // made it this far in the decision processing then we
                        // know there is at least one entry in the
                        // content_types_provided vector, but I want to confirm
                        // this is absolutlely the case.
                        r.content_types_provided(state).first().unwrap().clone()
                    });
                set_response_header(
                    state,
                    ContentType(Mime::clone(&content_type)),
                );
                let response_body = body_fn(req);
                set_response_body(state, response_body);
            }
            _ => (),
        };
        if let Some(etag) = r.generate_etag(state, req) {
            set_response_header(state, ETag(etag));
        }
        if let Some(modified) = r.last_modified(state) {
            set_response_header(state, LastModified(modified));
        }
        halt_with_response(StatusCode::Ok, state)
    }
}

fn o16<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "o16");
    match req.method() {
        Method::Put => o14(r, req, state),
        _ => o17(r, req, state),
    }
}

fn o17<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "o17");
    match req.method() {
        Method::Patch => {
            let accepted = r.patch_content_types_accepted(state);
            let result = req
                .headers()
                .get::<ContentType>()
                .and_then(|ct_hdr| map_content_media::<()>(accepted, ct_hdr));
            match result {
                Some(action) => {
                    action(req);
                    o20(r, req, state)
                }
                None => halt(StatusCode::UnsupportedMediaType, state),
            }
        }
        _ => o18(r, req, state),
    }
}

fn o14<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "o14");
    if r.is_conflict(state) {
        halt(StatusCode::Conflict, state)
    } else {
        let accepted = r.content_types_accepted(state);
        let result = req
            .headers()
            .get::<ContentType>()
            .and_then(|ct_hdr| map_content_media::<()>(accepted, ct_hdr));
        match result {
            Some(action) => {
                action(req);
                p11(r, req, state)
            }
            None => halt(StatusCode::UnsupportedMediaType, state),
        }
    }
}

// ------------------------------------------------------------------------------
// -- P column
// ------------------------------------------------------------------------------

fn p11<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "p11");
    if req.headers().has::<Location>() {
        halt(StatusCode::Created, state)
    } else {
        o20(r, req, state)
    }
}

fn p03<R, S>(r: &R, req: &Request, state: &mut S) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    trace(state, "p03");
    if r.is_conflict(state) {
        halt(StatusCode::Conflict, state)
    } else {
        let accepted = r.content_types_accepted(state);
        let result = req
            .headers()
            .get::<ContentType>()
            .and_then(|ct_hdr| map_content_media::<()>(accepted, ct_hdr));
        match result {
            Some(action) => {
                action(req);
                p11(r, req, state)
            }
            None => halt(StatusCode::UnsupportedMediaType, state),
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
    content_type: &ContentType,
) -> Option<fn(&Request) -> T> {
    let mut action_match = None;

    // Iterate through all of the provided Content-Types for the
    // resource and look for a match.
    for (ct_hdr, action) in &provided {
        if ct_hdr == &content_type.0 {
            action_match = Some(*action);
            break;
        }
    }
    action_match
}

/// Matches a list of server-side resource options against a quality-marked list
/// of client-side preferences.
fn map_accept_media(
    provided: Vec<(Mime, fn(&Request) -> Body)>,
    accept: &Accept,
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
                if (a_hdr.item == mime::STAR_STAR
                    && a_hdr.quality > match_quality)
                    || (a_hdr.item.type_() == ct_hdr.type_()
                        && a_hdr.quality > match_quality
                        && (a_hdr.item.subtype() == ct_hdr.subtype()
                            || a_hdr.item.subtype() == mime::STAR))
                {
                    type_match = Some((ct_hdr.clone(), *body_fn));
                    match_quality = a_hdr.quality;
                }
            }
        }
    }
    type_match
}

fn append_request_path(req: &Request, path_segments: &[String]) -> String {
    let path_suffix: String = path_segments
        .iter()
        .cloned()
        .intersperse(",".to_string())
        .collect();
    [req.path(), &path_suffix].concat()
}

fn create<R, S>(
    r: &R,
    req: &Request,
    state: &mut S,
    path_segments: &[String],
) -> Option<()>
where
    R: Webmachine,
    S: HasAirshipState,
{
    let location = append_request_path(req, path_segments);
    set_response_header(state, Location::new(location));
    let accepted = r.content_types_accepted(state);
    req.headers()
        .get::<ContentType>()
        .and_then(|ct_hdr| map_content_media::<()>(accepted, ct_hdr))
        .and_then(|action| {
            action(req);
            Some(())
        })
}

fn process_post_action<R, S>(
    r: &R,
    req: &Request,
    state: &mut S,
    pr: PostResponse,
) -> BoxedFuture
where
    R: Webmachine,
    S: HasAirshipState,
{
    match pr {
        PostResponse::PostCreate(ref path_segments) => {
            match create(r, req, state, path_segments) {
                Some(()) => p11(r, req, state),
                None => halt(StatusCode::UnsupportedMediaType, state),
            }
        }
        PostResponse::PostCreateRedirect(ref path_segments) => {
            match create(r, req, state, path_segments) {
                Some(()) => halt(StatusCode::SeeOther, state),
                None => halt(StatusCode::UnsupportedMediaType, state),
            }
        }
        PostResponse::PostProcess(accepted) => {
            let result = req
                .headers()
                .get::<ContentType>()
                .and_then(|ct_hdr| map_content_media::<()>(accepted, ct_hdr));
            match result {
                Some(action) => {
                    action(req);
                    p11(r, req, state)
                }
                None => halt(StatusCode::UnsupportedMediaType, state),
            }
        }
        PostResponse::PostProcessRedirect(accepted) => {
            let result =
                req.headers().get::<ContentType>().and_then(|ct_hdr| {
                    map_content_media::<String>(accepted, ct_hdr)
                });
            match result {
                Some(action) => {
                    let location = action(req);
                    set_response_header(state, Location::new(location));
                    halt(StatusCode::SeeOther, state)
                }
                None => halt(StatusCode::UnsupportedMediaType, state),
            }
        }
    }
}
