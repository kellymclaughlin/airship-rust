#![allow(clippy::type_complexity)]

use hyper::{Body, Method, Request, Uri};
use hyper::header::*;
use mime;
use mime::Mime;

use webmachine_derive::*;

use crate::types::HasAirshipState;

pub trait Webmachine {
    // Whether to allow HTTP POSTs to a missing resource. Default: false.
    fn allow_missing_post<S: HasAirshipState>(&self, _state: &mut S) -> bool {
        false
    }

    /*
     * The set of HTTP methods that this resource allows. Default: @GET@ and
     * @HEAD@. If a request arrives with an HTTP method not included herein,
     * @501 Not Implemented@ is returned.
     */
    fn allowed_methods<S: HasAirshipState>(&self, _state: &mut S) -> Vec<Method> {
        vec![Method::Get, Method::Head, Method::Options]
    }

    /*
     * An association list of 'MediaType's and 'Webmachine' actions that
     * correspond to the accepted @Content-Type@ values that this resource
     * can accept in a request body. If a @Content-Type@ header is present
     * but not accounted for in 'content_types_accepted', processing will
     * halt with @415 Unsupported Media Type@. Otherwise, the corresponding
     * 'Webmachine' action will be executed and processing will continue.
     */
    fn content_types_accepted<S: HasAirshipState>(&self, _state: &mut S) -> Vec<(Mime, fn(&Request))> {
        vec![]
    }

    /*
     * An association list of 'Mime' values and 'ResponseBody' values. The
     * response will be chosen by looking up the 'Mime' that most closely
     * matches the @Accept@ header. Should there be no match, processing
     * will halt with @406 Not Acceptable@.
     */
    fn content_types_provided<S: HasAirshipState>(&self, _state: &mut S) -> Vec<(Mime, fn(&Request) -> Body)> {
        vec![(mime::TEXT_PLAIN, |_x:&Request| Body::empty())]
    }

    /*
     * When a @DELETE@ request is enacted (via a @True@ value returned from
     * 'delete_resource'), a @False@ value returns a @202 Accepted@ response.
     * Returning @True@ will continue processing, usually ending up with a
     * @204 No Content@ response. Default: False.
     */
    fn delete_completed<S: HasAirshipState>(&self, _state: &mut S) -> bool {
        false
    }

    /*
     * When processing a @DELETE@ request, a @True@ value allows processing
     *  to continue. Returns @500 Forbidden@ if False. Default: false.
     */
    fn delete_resource<S: HasAirshipState>(&self, _state: &mut S, _req: &Request) -> bool {
        false
    }

    // Returns @413 Request Entity Too Large@ if true. Default: false.
    fn entity_too_large<S: HasAirshipState>(&self, _state: &mut S, _req: &Request) -> bool {
        false
    }

    /*
     * Checks if the given request is allowed to access this resource.
     * Returns @403 Forbidden@ if true. Default: false.
     */
    fn forbidden<S: HasAirshipState>(&self, _state: &mut S, _req: &Request) -> bool {
        false
    }

    /*
     * If this returns a non-'Nothing' 'ETag', its value will be added to
     * every HTTP response in the @ETag:@ field.
     */
    fn generate_etag<S: HasAirshipState>(&self, _state: &mut S, _req: &Request) -> Option<EntityTag> {
        None
    }

    // Checks if this resource has actually implemented a handler for a given HTTP method.
    // Returns @501 Not Implemented@ if false. Default: true.
    fn implemented<S: HasAirshipState>(&self, _state: &mut S) -> bool {
        true
    }

    // Returns @401 Unauthorized@ if false. Default: true.
    fn is_authorized<S: HasAirshipState>(&self, _state: &mut S, _req: &Request) -> bool {
        true
    }

    /*
     * When processing @PUT@ requestsfn a @True@ value returned here will
     * halt processing with a @409 Conflict@.
     */
    fn is_conflict<S: HasAirshipState>(&self, _state: &mut S) -> bool {
        false
    }

    // Returns @415 Unsupported Media Type@ if false. We recommend you use the 'contentTypeMatches' helper functionfn which accepts a list of
    // 'MediaType' valuesfn so as to simplify proper MIME type handling. Default: true.
    fn known_content_type<S: HasAirshipState>(&self, _state: &mut S, _req: &Request) -> bool {
        true
    }

    // In the presence of an @If-Modified-Since@ headerfn returning a @Just@ value from 'lastModifed' allows
    // the server to halt with @304 Not Modified@ if appropriate.
    fn last_modified<S: HasAirshipState>(&self, _state: &mut S) -> Option<HttpDate> {
        None
    }

    /*
     * If an @Accept-Language@ value is present in the HTTP request, and this
     * function returns @False@, processing will halt with
     * @406 Not Acceptable@.
     */
    fn language_available<H: Header, S: HasAirshipState>(&self, _state: &mut S, _accept_lang_header: &H) -> bool {
        true
    }

    // Returns @400 Bad Request@ if true. Default: false.
    fn malformed_request<S: HasAirshipState>(&self, _state: &mut S, _req: &Request) -> bool {
        false
    }

    /*
     * When processing a resource for which 'resourceExists' returned
     * @False@, returning a @Just@ value halts with a
     * @301 Moved Permanently@ response. The contained 'String' will be
     * added to the HTTP response under the @Location:@ header.
     */
    fn moved_permanently<S: HasAirshipState>(&self, _state: &mut S) -> Option<String> {
        None
    }

    // Like 'moved_permanently'fn except with a @307 Moved Temporarily@ response.
    fn moved_temporarily<S: HasAirshipState>(&self, _state: &mut S) -> Option<String> {
        None
    }

    /*
     * When handling a @PUT@ requestfn returning @True@ here halts
     * processing with @300 Multiple Choices@. Default: False.
     */
    fn multiple_choices<S: HasAirshipState>(&self, _state: &mut S) -> bool {
        false
    }

    /*
     * As 'contentTypesAccepted', but checked and executed specifically in
     * the case of a PATCH request.
     */
    fn patch_content_types_accepted<S: HasAirshipState>(&self, _state: &mut S) -> Vec<(Mime, fn(&Request))> {
        vec![]
    }

    /*
     * When processing a request for which 'resource_exists' returned
     * @False@, returning @True@ here allows the 'moved_permanently' and
     * 'moved_temporarily' functions to process the request.
     */
    fn previously_existed<S: HasAirshipState>(&self, _state: &mut S) -> bool {
        false
    }

    /* When handling @POST@ requests the value returned determines whether
     * to treat the request as a @PUT@, a @PUT@ and a redirect or a plain
     * @POST@. See the documentation for 'PostResponse' for more information.
     * The default implemetation returns a 'PostProcess' with an empty
     * handler.
     */
    fn process_post<S: HasAirshipState>(&self, _state: &mut S, _req: &Request) -> PostResponse {
        PostResponse::PostProcess(vec![])
    }

    /*
     * Does the resource at this path exist?
     * Returning false from this usually entails a @404 Not Found@ response.
     * (If 'allowMissingPost' returns @True@ or an @If-Match: *@ header is
     * present, it may not).
     */
    fn resource_exists<S: HasAirshipState>(&self, _state: &mut S) -> bool {
        true
    }

    // Returns @503 Service Unavailable@ if false. Default: true.
    fn service_available<S: HasAirshipState>(&self, _state: &mut S) -> bool {
        true
    }

    // Returns @414 Request URI Too Long@ if true. Default: false.
    fn uri_too_long<S: HasAirshipState>(&self, _state: &mut S, _uri: &Uri) -> bool {
        false
    }

    // Returns @501 Not Implemented@ if false. Default: true.
    fn valid_content_headers<S: HasAirshipState>(&self, _state: &mut S, _req: &Request) -> bool {
        true
    }
}

// #[derive(Clone)]
#[derive(Clone, Webmachine)]
pub struct Resource;

// impl Webmachine for Resource {}

/// Used when processing POST requests so as to handle the outcome of the binary
/// decisions between handling a POST as a create request and whether to
/// redirect after the POST is done.  Credit for this idea goes to Richard
/// Wallace (purefn) on Webcrank.
///
/// For processing the POST, an association list of `Mime`s and `Webmachine`
/// actions are required that correspond to the accepted `Content-Type` values
/// that this resource can accept in a request body.  If a `Content-Type` header
/// is present but not accounted for, processing will halt with `415 Unsupported
/// Media Type`.
pub enum PostResponse
{
    /// Treat this request as a `PUT`.
    PostCreate(Vec<String>),
    /// Treat this request as a `PUT`, then redirect.
    PostCreateRedirect(Vec<String>),
    /// Process as a `POST`, but don't redirect.
    PostProcess(Vec<(Mime, fn(&Request))>),
    /// Process and redirect.
    PostProcessRedirect(Vec<(Mime, fn(&Request) -> String)>)
}
