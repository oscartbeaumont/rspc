use httpz::{
    http::{Response, StatusCode},
    ws::WebsocketUpgrade,
    HttpResponse,
};

use crate::internal::exec::{Connection, ConnectionTask, Executor, TokioRuntime};

use super::TCtxFunc;

pub(crate) fn handle_websocket<TCtx, TCtxFn, TCtxFnMarker>(
    executor: Executor<TCtx, TokioRuntime>,
    ctx_fn: TCtxFn,
    req: httpz::Request,
) -> impl HttpResponse
where
    TCtx: Clone + Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    if !req.server().supports_websockets() {
        #[cfg(feature = "tracing")]
        tracing::debug!("Websocket are not supported on your webserver!");

        // TODO: Make this error be picked up on the frontend and expose it with a logical name
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(vec![])?);
    } else {
        #[cfg(feature = "tracing")]
        tracing::debug!("Accepting websocket connection");
    }

    // TODO: Remove need for `_internal_dangerously_clone`
    let ctx = match ctx_fn.exec(req._internal_dangerously_clone(), None) {
        Ok(v) => v,
        Err(_err) => {
            #[cfg(feature = "tracing")]
            tracing::error!("Error executing context function: {}", _err);

            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(vec![])?);
        }
    };

    let cookies = req.cookies(); // TODO: Reorder args of next func so cookies goes first
    WebsocketUpgrade::from_req_with_cookies(req, cookies, move |_, socket| {
        ConnectionTask::<TokioRuntime, TCtx, _, _, _, _>::new(
            Connection::new(ctx, executor),
            socket,
        )
    })
    .into_response()
}