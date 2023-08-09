use std::{
    net::{Ipv6Addr, SocketAddr},
    path::PathBuf,
    time::Duration, sync::atomic::AtomicU16, task::{Poll, Context}, pin::Pin,
};

use futures::Stream;
use async_stream::stream;
use axum::routing::get;
use rspc::{integrations::httpz::Request, ErrorCode, ExportConfig, Rspc, Blob};
use tokio::{time::sleep, fs::File, io::BufReader};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[derive(Clone)]
struct Ctx {
    x_demo_header: Option<String>,
}

const R: Rspc<Ctx> = Rspc::new();

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = R
        .router()
        .procedure(
            "version",
            R
                .with(|mw, ctx| async move {
                    mw.next(ctx).map(|resp| async move {
                        println!("Client requested version '{}'", resp);
                        resp
                    })
                })
                .with(|mw, ctx| async move { mw.next(ctx) })
                .query(|_, _: ()| {
                    info!("Client requested version");
                    env!("CARGO_PKG_VERSION")
                }),
        )
        .procedure(
            "X-Demo-Header",
            R.query(|ctx, _: ()| ctx.x_demo_header.unwrap_or_else(|| "No header".to_string())),
        )
        .procedure("echo", R.query(|_, v: String| v))
        .procedure(
            "error",
            R.query(|_, _: ()| {
                Err(rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    "Something went wrong".into(),
                )) as Result<String, rspc::Error>
            }),
        )
        .procedure(
            "error",
            R.mutation(|_, _: ()| {
                Err(rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    "Something went wrong".into(),
                )) as Result<String, rspc::Error>
            }),
        )
        .procedure(
            "transformMe",
            R.query(|_, _: ()| "Hello, world!".to_string()),
        )
        .procedure(
            "sendMsg",
            R.mutation(|_, v: String| {
                println!("Client said '{}'", v);
                v
            }),
        )
        .procedure(
            "pings",
            R.subscription(|_, _: ()| {
                println!("Client subscribed to 'pings'");
                stream! {
                    yield "start".to_string();
                    for i in 0..5 {
                        info!("Sending ping {}", i);
                        yield i.to_string();
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }),
        )
        .procedure("errorPings", R.subscription(|_ctx, _args: ()| {
            stream! {
                for _ in 0..5 {
                    yield Ok("ping".to_string());
                    sleep(Duration::from_secs(1)).await;
                }
                yield Err(rspc::Error::new(ErrorCode::InternalServerError, "Something went wrong".into()));
            }
        }))
        .procedure(
            "testSubscriptionShutdown",
            R.subscription({
                static COUNT: AtomicU16 = AtomicU16::new(0);
                |_, _: ()| {
                    let id = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    pub struct HandleDrop {
                        id: u16,
                        send: bool,
                    }

                    impl Stream for HandleDrop {
                        type Item = u16;
    
                        fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                            if self.send {
                                Poll::Pending
                            } else {
                                self.send = true;
                                Poll::Ready(Some(self.id))
                            }
                        }
                    }
    
                    impl Drop for HandleDrop {
                        fn drop(&mut self) {
                            println!("Dropped subscription with id {}", self.id);
                        }
                    }

                    HandleDrop { id, send: false }
                }
            }),
        )
        // TODO: This is an unstable feature and should be used with caution!
        .procedure("serveFile", R.query(|_, _: ()| async move {
            let file = File::open("./demo.json").await.unwrap();

            // TODO: What if type which is `futures::Stream` + `tokio::AsyncRead`???

            Blob(BufReader::new(file))
        }))
        .build()
        .unwrap()
        .arced(); // This function is a shortcut to wrap the router in an `Arc`.

    router
        .export_ts(ExportConfig::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bindings.ts"),
        ))
        .unwrap();

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

    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, 4000));
    println!("listening on http://{}/rspc/version", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
