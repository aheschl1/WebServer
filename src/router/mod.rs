use std::future::Future;
use http_body_util::combinators::BoxBody;
use hyper::{body::Bytes, Method, Request, Response};
use hyper::Error;

/**
 * Create a service that routes requests to different handlers based on the request method.
 * 
 * # Arguments
 * * `not_implemented` - The handler to call when the request method is not implemented.
 * * `get` - The handler to call when the request method is GET.
 * * `post` - The handler to call when the request method is POST.
 * * `put` - The handler to call when the request method is PUT.
 * * `delete` - The handler to call when the request method is DELETE.
 * * `patch` - The handler to call when the request method is PATCH.
 * * `head` - The handler to call when the request method is HEAD.
 * * `connect` - The handler to call when the request method is CONNECT.
 * * `options` - The handler to call when the request method is OPTIONS.
 * * `trace` - The handler to call when the request method is TRACE.
 * * `other` - The handler to call when the request method is not one of the above.
 */
pub fn routed_service<F, Fut>(
    not_implemented: F,
    get: Option<F>,
    post: Option<F>,
    put: Option<F>,
    delete: Option<F>,
    patch: Option<F>,
    head: Option<F>,
    connect: Option<F>,
    options: Option<F>,
    trace: Option<F>
) -> impl Fn(Request<hyper::body::Incoming>) -> Fut + Copy + Send + Sync + 'static
where
    F: Copy + Fn(Request<hyper::body::Incoming>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response<BoxBody<Bytes, Error>>, Error>> + Send + 'static,
{
    fn inner<F, Fut>(
        not_implemented: F,
        get: Option<F>,
        post: Option<F>,
        put: Option<F>,
        delete: Option<F>,
        patch: Option<F>,
        head: Option<F>,
        connect: Option<F>,
        options: Option<F>,
        trace: Option<F>,
        request: Request<hyper::body::Incoming>,
    ) -> Fut
    where
        F: Copy + Fn(Request<hyper::body::Incoming>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response<BoxBody<Bytes, Error>>, Error>> + Send + 'static,
    {
        match request.method() {
            &Method::GET => {
                if let Some(f) = get {
                    return f(request);
                } else {
                    return not_implemented(request);
                }
            }
            &Method::POST => {
                if let Some(f) = post {
                    return f(request);
                } else {
                    return not_implemented(request);
                }
            }
            &Method::PUT => {
                if let Some(f) = put {
                    return f(request);
                } else {
                    return not_implemented(request);
                }
            }
            &Method::DELETE => {
                if let Some(f) = delete {
                    return f(request);
                } else {
                    return not_implemented(request);
                }
            }
            &Method::PATCH => {
                if let Some(f) = patch {
                    return f(request);
                } else {
                    return not_implemented(request);
                }
            }
            &Method::HEAD => {
                if let Some(f) = head {
                    return f(request);
                } else {
                    return not_implemented(request);
                }
            }
            &Method::CONNECT => {
                if let Some(f) = connect {
                    return f(request);
                } else {
                    return not_implemented(request);
                }
            }
            &Method::OPTIONS => {
                if let Some(f) = options {
                    return f(request);
                } else {
                    return not_implemented(request);
                }
            }
            &Method::TRACE => {
                if let Some(f) = trace {
                    return f(request);
                } else {
                    return not_implemented(request);
                }
            }
            _ => not_implemented(request)
        }
    }

    move |request: Request<hyper::body::Incoming>| {
        inner(
            not_implemented, get, post, put, delete, patch, head, connect, options, trace, request,
        )
    }
}