use std::{path::PathBuf, time::Duration};

use async_stream::stream;
use axum::routing::get;
use rspc::{alpha::Rspc, integrations::httpz::Request, Config};
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};

struct Ctx {
    x_demo_header: Option<String>,
}

const R: Rspc<Ctx> = Rspc::new();

#[tokio::main]
async fn main() {
    let router = R
        .router()
        .procedure("version", R.query(|_, _: ()| env!("CARGO_PKG_VERSION")))
        .procedure(
            "version1",
            R.with(|mw, ctx| async move {
                println!("MW ONE");
                mw.next(ctx)
            })
            .query(|_, _: ()| env!("CARGO_PKG_VERSION")),
        )
        .procedure(
            "version2",
            R.with(|mw, ctx| async move {
                println!("MW ONE");
                mw.next(ctx)
            })
            .with(|mw, ctx| async move {
                println!("MW TWO");
                mw.next(ctx)
            })
            .query(|_, _: ()| env!("CARGO_PKG_VERSION")),
        )
        .procedure(
            "version3",
            R.with(|mw, ctx| async move {
                println!("MW ONE");
                mw.next(ctx)
            })
            .with(|mw, ctx| async move {
                println!("MW TWO");
                mw.next(ctx)
            })
            .with(|mw, ctx| async move {
                println!("MW THREE");
                mw.next(ctx)
            })
            .query(|_, _: ()| env!("CARGO_PKG_VERSION")),
        )
        .procedure(
            "version4",
            R.with(|mw, ctx| async move {
                println!("MW ONE");
                mw.next(ctx).resp(|result| async move {
                    println!("MW ONE RESULT: {result:?}");
                    result
                })
            })
            .query(|_, _: ()| env!("CARGO_PKG_VERSION")),
        )
        .compat()
        .arced();

    // let router =
    //     rspc::Router::<Ctx>::new()
    //         .config(Config::new().export_ts_bindings(
    //             PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bindings.ts"),
    //         ))
    //         .query("version", |t| t(|_, _: ()| env!("CARGO_PKG_VERSION")))
    //         .query("X-Demo-Header", |t| {
    //             t(|ctx, _: ()| {
    //                 ctx.x_demo_header
    //                     .clone()
    //                     .unwrap_or_else(|| "No header".to_string())
    //             })
    //         })
    //         .query("echo", |t| t(|_, v: String| v))
    //         .query("error", |t| {
    //             t(|_, _: ()| {
    //                 Err(rspc::Error::new(
    //                     rspc::ErrorCode::InternalServerError,
    //                     "Something went wrong".into(),
    //                 )) as Result<String, rspc::Error>
    //             })
    //         })
    //         .mutation("error", |t| {
    //             t(|_, _: ()| {
    //                 Err(rspc::Error::new(
    //                     rspc::ErrorCode::InternalServerError,
    //                     "Something went wrong".into(),
    //                 )) as Result<String, rspc::Error>
    //             })
    //         })
    //         .query("transformMe", |t| t(|_, _: ()| "Hello, world!".to_string()))
    //         .mutation("sendMsg", |t| {
    //             t(|_, v: String| {
    //                 println!("Client said '{}'", v);
    //                 v
    //             })
    //         })
    //         .subscription("pings", |t| {
    //             t(|_ctx, _args: ()| {
    //                 stream! {
    //                     println!("Client subscribed to 'pings'");
    //                     for i in 0..5 {
    //                         println!("Sending ping {}", i);
    //                         yield "ping".to_string();
    //                         sleep(Duration::from_secs(1)).await;
    //                     }
    //                 }
    //             })
    //         })
    //         // TODO: Results being returned from subscriptions
    //         // .subscription("errorPings", |t| t(|_ctx, _args: ()| {
    //         //     stream! {
    //         //         for i in 0..5 {
    //         //             yield Ok("ping".to_string());
    //         //             sleep(Duration::from_secs(1)).await;
    //         //         }
    //         //         yield Err(rspc::Error::new(ErrorCode::InternalServerError, "Something went wrong".into()));
    //         //     }
    //         // }))
    //         .build()
    //         .arced(); // This function is a shortcut to wrap the router in an `Arc`.

    // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .nest(
            "/rspc",
            router
                .clone()
                .endpoint(|req: Request| {
                    println!("Client requested operation '{}'", req.uri().path());
                    Ctx {
                        x_demo_header: req
                            .headers()
                            .get("X-Demo-Header")
                            .map(|v| v.to_str().unwrap().to_string()),
                    }
                })
                .axum(),
        )
        .layer(cors);

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc/version", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
