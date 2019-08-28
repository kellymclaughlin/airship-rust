use std::rc::Rc;

use futures::Future;
use hyper::{Body, Method, Request, Response, StatusCode};
use hyper::header::*;
use mime::Mime;

use crate::resource::Webmachine;
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
        Response::new().with_status(status_code).with_header(hdr),
    ))
}


///////////////////////////////////////////////////////////////////////////////
// B column
///////////////////////////////////////////////////////////////////////////////

fn b13<R: Webmachine>(r: &R, _req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "b13");
    match r.service_available() {
        true => b12(r, _req, state),
        false => halt(StatusCode::ServiceUnavailable)
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

fn map_accept_media(_provided: Vec<(Mime, Box<Fn(&Request) -> Body>)>, _accept: &Vec<QualityItem<Mime>>) -> Option<(Mime, Box<Fn(&Request) -> Body>)> {
    Some((mime::TEXT_PLAIN, Box::new(|_| { Body::from("ok") })))
}

fn c04<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState, accept_header: &Accept) -> BoxedFuture {
    trace(state, "c04");
    let provided = r.content_types_provided();
    let result = Rc::new(map_accept_media(provided, &accept_header.0));
    // let result = map_accept_media(provided, &accept_header.0);
    match *result {
        Some((ref accept_type, ref _resource)) => {
            // let accept_type_clone = Rc::new(accept_type);
            set_response_header(state, ContentType(Mime::clone(&accept_type)));
            // set_matched_content_type(state, Rc::clone(&result));
            let r_clone = Rc::clone(&result);
            state.matched_content_type = r_clone;
            d04(r, req, state)
        },
        None => halt(StatusCode::NotAcceptable)
    }
}
// c04 r@Resource{..} = do
//     trace "c04"
//     req <- lift request
//     provided <- lift contentTypesProvided
//     let reqHeaders = requestHeaders req
//         result = do
//             acceptStr <- lookup HTTP.hAccept reqHeaders
//             (acceptTyp, resource) <- mapAcceptMedia provided' acceptStr
//             Just (acceptTyp, resource)
//             where
//                 -- this is so that in addition to getting back the resource
//                 -- that we match, we also return the content-type provided
//                 -- by that resource.
//                 provided' = map dupContentType provided
//                 dupContentType (a, b) = (a, (a, b))

//     case result of
//       Nothing -> lift $ halt HTTP.status406
//       Just res -> do
//         modify (\fs -> fs { _contentType = Just res })
//         d04 r

fn c03<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "c03");
    match req.headers().get::<Accept>() {
        Some(ahdr) => c04(r, req, state, &ahdr),
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

fn n11<R: Webmachine>(_r: &R, _req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "n11");
    // TODO: Implement POST handling
    halt(StatusCode::Created)
}
// n11 r@Resource{..} = trace "n11" >> lift processPost >>= flip processPostAction r

// create :: Monad m => [Text] -> Resource m -> FlowStateT m ()
// create ts Resource{..} = do
//     loc <- lift (appendRequestPath ts)
//     lift (addResponseHeader ("Location", loc))
//     lift contentTypesAccepted >>= negotiateContentTypesAccepted

// processPostAction :: Monad m => PostResponse m -> Flow  m
// processPostAction (PostCreate ts) r = do
//     create ts r
//     p11 r
// processPostAction (PostCreateRedirect ts) r = do
//     create ts r
//     lift $ halt HTTP.status303
// processPostAction (PostProcess accepted) r = do
//     negotiateContentTypesAccepted accepted >> p11 r
// processPostAction (PostProcessRedirect accepted) _r = do
//     locBs <- negotiateContentTypesAccepted accepted
//     lift $ addResponseHeader ("Location", locBs)
//     lift $ halt HTTP.status303

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


fn o18<R: Webmachine>(r: &R, _req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "o18");
    match r.multiple_choices() {
        true  => halt(StatusCode::MultipleChoices),
        false => {
            //  -- TODO: set etag, expiration, etc. headers
            //  req <- lift request
            //  let getOrHead = [ HTTP.methodGet
            //                  , HTTP.methodHead
            //                  ]
            //  when (requestMethod req `elem` getOrHead) $ do
            //      m <- _contentType <$> get
            //      (cType, body) <- case m of
            //          Nothing -> do
            //              provided <- lift contentTypesProvided
            //              return (head provided)
            //          Just (cType, body) ->
            //              return (cType, body)
            //      b <- lift body
            //      lift $ putResponseBody b
            //      lift $ addResponseHeader ("Content-Type", renderHeader cType)
            //      writeCacheTags r
            // set_response_header(state, AirshipTrace(trace));
            halt_with_response(StatusCode::Ok, state)
        }
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
//        else lift patchContentTypesAccepted >>= negotiateContentTypesAccepted >> o20 r
            o20(r, req, state)
        },
        _             => o18(r, req, state)
    }
}

fn o14<R: Webmachine>(r: &R, req: &Request, state: &mut RequestState) -> BoxedFuture {
    trace(state, "o14");
    match r.is_conflict() {
        true  => halt(StatusCode::Conflict),
        false => {
//         else lift contentTypesAccepted >>= negotiateContentTypesAccepted >> p11 r
            p11(r, req, state)
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
    match r.is_conflict() {
        true => halt(StatusCode::Conflict),
        false =>
        // else lift contentTypesAccepted >>= negotiateContentTypesAccepted >> p11 r
            p11(r, req, state)
    }
}
