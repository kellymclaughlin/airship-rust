use hyper::Uri;
use hyper::Method;
use hyper::Method::{Get, Head, Options};
use hyper::header::EntityTag;
use hyper::server::{Request};

use mime;
use mime::Mime;

// type ErrorResponses m = Monad m => Map HTTP.Status [(MediaType, Webmachine m ResponseBody)]

// type ErrorResponses = Map HTTP.Status [(MediaType, ResponseBody)]
pub type ErrorResponses = String;
pub type ResponseBody = String;

pub trait Webmachine {
    // Whether to allow HTTP POSTs to a missing resource. Default: false.
    fn allow_missing_post(&self) -> bool {
        false
    }

    /*
     * The set of HTTP methods that this resource allows. Default: @GET@ and
     * @HEAD@. If a request arrives with an HTTP method not included herein,
     * @501 Not Implemented@ is returned.
     */
    fn allowed_methods(&self) -> Vec<Method> {
        vec![Get, Head, Options]
    }

    /*
     * An association list of 'MediaType's and 'Webmachine' actions that
     * correspond to the accepted @Content-Type@ values that this resource
     * can accept in a request body. If a @Content-Type@ header is present
     * but not accounted for in 'contentTypesAccepted', processing will
     * halt with @415 Unsupported Media Type@. Otherwise, the corresponding
     * 'Webmachine' action will be executed and processing will continue.
     */
    fn content_types_accepted(&self) -> Vec<(Mime, ())> {
        vec![]
    }

    /*
     * An association list of 'Mime' values and 'ResponseBody' values. The
     * response will be chosen by looking up the 'Mime' that most closely
     * matches the @Accept@ header. Should there be no match, processing
     * will halt with @406 Not Acceptable@.
     */
    fn content_types_provided(&self) -> Vec<(Mime, ResponseBody)> {
        vec![(mime::TEXT_PLAIN, String::from(""))]
    }

    /*
     * When a @DELETE@ request is enacted (via a @True@ value returned from
     * 'deleteResource'), a @False@ value returns a @202 Accepted@ response.
     * Returning @True@ will continue processing, usually ending up with a
     * @204 No Content@ response. Default: False.
     */
    fn delete_completed(&self) -> bool {
        false
    }

    /*
     * When processing a @DELETE@ request, a @True@ value allows processing
     *  to continue. Returns @500 Forbidden@ if False. Default: false.
     */
    fn delete_resource(&self) -> bool {
        false
    }

    // Returns @413 Request Entity Too Large@ if true. Default: false.
    fn entity_too_large(&self) -> bool {
        false
    }

    /*
     * Checks if the given request is allowed to access this resource.
     * Returns @403 Forbidden@ if true. Default: false.
     */
    fn forbidden(&self) -> bool {
        false
    }

    /*
     * If this returns a non-'Nothing' 'ETag', its value will be added to
     * every HTTP response in the @ETag:@ field.
     */
    fn generate_etag(&self) -> Option<EntityTag> {
        None
    }

    // Checks if this resource has actually implemented a handler for a given HTTP method.
    // Returns @501 Not Implemented@ if false. Default: true.
    fn implemented(&self) -> bool {
        true
    }

    // Returns @401 Unauthorized@ if false. Default: true.
    fn is_authorized(&self) -> bool {
        true
    }

    /*
     * When processing @PUT@ requestsfn a @True@ value returned here will
     * halt processing with a @409 Conflict@.
     */
    fn is_conflict(&self) -> bool {
        false
    }

    // Returns @415 Unsupported Media Type@ if false. We recommend you use the 'contentTypeMatches' helper functionfn which accepts a list of
    // 'MediaType' valuesfn so as to simplify proper MIME type handling. Default: true.
    fn known_content_type(&self) -> bool {
        true
    }

    // In the presence of an @If-Modified-Since@ headerfn returning a @Just@ value from 'lastModifed' allows
    // the server to halt with @304 Not Modified@ if appropriate.
    fn last_modified(&self) -> Option<String> {
        //TODO: use time type equiv (Maybe UTCTime)
        None
    }

    /*
     * If an @Accept-Language@ value is present in the HTTP request, and this
     * function returns @False@, processing will halt with
     * @406 Not Acceptable@.
     */
    fn language_available(&self) -> bool {
        true
    }

    // Returns @400 Bad Request@ if true. Default: false.
    fn malformed_request(&self, _req: &Request) -> bool {
        false
    }

    /*
     * When processing a resource for which 'resourceExists' returned
     * @False@, returning a @Just@ value halts with a
     * @301 Moved Permanently@ response. The contained 'String' will be
     * added to the HTTP response under the @Location:@ header.
     */
    fn moved_permanently(&self) -> Option<String> {
        None
    }

    // Like 'movedPermanently'fn except with a @307 Moved Temporarily@ response.
    fn moved_temporarily(&self) -> Option<String> {
        None
    }

    /*
     * When handling a @PUT@ requestfn returning @True@ here halts
     * processing with @300 Multiple Choices@. Default: False.
     */
    fn multiple_choices(&self) -> bool {
        false
    }

    /*
     * As 'contentTypesAccepted', but checked and executed specifically in
     * the case of a PATCH request.
     */
    fn patch_content_types_accepted(&self) -> Vec<(Mime, ())> {
        vec![]
    }

    /*
     * When processing a request for which 'resourceExists' returned
     * @False@, returning @True@ here allows the 'movedPermanently' and
     * 'movedTemporarily' functions to process the request.
     */
    fn previously_existed(&self) -> bool {
        false
    }

    /* When handling @POST@ requestsfn the value returned determines whether
     * to treat the request as a @PUT@, a @PUT@ and a redirectfn or a plain
     * @POST@. See the documentation for 'PostResponse' for more information.
     * The default implemetation returns a 'PostProcess' with an empty
     * handler.
     */
    fn process_post(&self) -> () {
        //TODO should return equiv of (PostResponse m)
        ()
    }

    /*
     * Does the resource at this path exist?
     * Returning false from this usually entails a @404 Not Found@ response.
     * (If 'allowMissingPost' returns @True@ or an @If-Match: *@ header is
     * present, it may not).
     */
    fn resource_exists(&self) -> bool {
        true
    }

    // Returns @503 Service Unavailable@ if false. Default: true.
    fn service_available(&self) -> bool {
        true
    }

    // Returns @414 Request URI Too Long@ if true. Default: false.
    fn uri_too_long(&self, _uri: &Uri) -> bool {
        false
    }

    // Returns @501 Not Implemented@ if false. Default: true.
    fn valid_content_headers(&self) -> bool {
        true
    }

    // Helper function to trace decision tree traversal
    fn trace(&mut self, _t: String) -> &Self {
        self
    }
}

pub struct Resource {
    pub error_responses: ErrorResponses,
    pub decision_trace: Vec<String>,
}

impl Webmachine for Resource {
    fn trace(&mut self, t: String) -> &Self {
        self.decision_trace.push(t);
        self
    }
}
