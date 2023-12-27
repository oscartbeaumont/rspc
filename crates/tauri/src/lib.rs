// //! Access rspc via the Tauri IPC bridge.
// #![warn(
//     clippy::all,
//     clippy::cargo,
//     clippy::unwrap_used,
//     clippy::panic,
//     clippy::todo,
//     clippy::panic_in_result_fn,
//     // missing_docs
// )]
// #![forbid(unsafe_code)]
// #![allow(clippy::module_inception)]
// #![cfg_attr(docsrs, feature(doc_cfg))]

// use std::{
//     collections::{hash_map::DefaultHasher, HashMap},
//     convert::Infallible,
//     hash::{Hash, Hasher},
//     marker::PhantomData,
//     sync::{Arc, Mutex},
// };

// use futures::{channel::mpsc, sink, StreamExt};
// use tauri::{
//     plugin::{Builder, TauriPlugin},
//     Runtime, Window, WindowEvent,
// };
// use tauri_specta::Event;

// use rspc_core::{
//     exec::{run_connection, IncomingMessage, Response, SinkAndStream},
//     Router2,
// };

// #[derive(Clone, Debug, serde::Deserialize, specta::Type, tauri_specta::Event)]
// struct Msg(serde_json::Value);

// #[derive(Clone, serde::Serialize, specta::Type, tauri_specta::Event)]
// #[specta(inline)]
// struct TransportResp(Vec<Response>);

// struct WindowManager<R, TCtxFn, TCtx> {
//     router: Arc<Router2<TCtx>>,
//     ctx_fn: TCtxFn,
//     windows: Mutex<HashMap<u64, mpsc::UnboundedSender<()>>>,
//     phantom: PhantomData<fn() -> R>,
// }

// impl<R, TCtxFn, TCtx> WindowManager<R, TCtxFn, TCtx>
// where
//     R: Runtime,
//     TCtx: Clone + Send + Sync + 'static,
//     TCtxFn: Fn(&Window<R>) -> TCtx + Send + Sync + 'static,
// {
//     pub fn new(ctx_fn: TCtxFn, router: Arc<Router2<TCtx>>) -> Arc<Self> {
//         Arc::new(Self {
//             router,
//             ctx_fn,
//             windows: Mutex::new(HashMap::new()),
//             phantom: PhantomData,
//         })
//     }

//     pub fn on_page_load(self: Arc<Self>, window: Window<R>) {
//         let mut hasher = DefaultHasher::new();
//         window.hash(&mut hasher);
//         let window_hash = hasher.finish();

//         #[allow(clippy::unwrap_used)] // TODO: Stop using unwrap
//         let mut windows = self.windows.lock().unwrap();
//         if let Some(shutdown_streams_tx) = windows.get(&window_hash) {
//             // Shutdown all subscriptions for the previously loaded page is there was one
//             // All the previous threads and stuff stays around though so we don't need to recreate it

//             shutdown_streams_tx.unbounded_send(()).ok();
//         } else {
//             let (clear_subscriptions_tx, clear_subscriptions_rx) = mpsc::unbounded();
//             windows.insert(window_hash, clear_subscriptions_tx);
//             drop(windows);

//             let (tx, rx) = mpsc::unbounded();

//             Msg::listen(&window, move |event| {
//                 tx.unbounded_send(IncomingMessage::Msg(Ok(event.payload.0)))
//                     .ok();
//             });

//             // passing in 'window' allows us to not clone it, with Unfold reusing the window from the previous iteration.
//             // less clones and happy lifetimes
//             let sink = sink::unfold(window.clone(), move |window, item| async move {
//                 TransportResp(item)
//                     .emit(&window)
//                     .map_err(|_err| {
//                         #[cfg(feature = "tracing")]
//                         tracing::error!("failed to emit JSON-RPC response: {}", _err);
//                     })
//                     .ok();

//                 Ok::<_, Infallible>(window)
//             });

//             tauri::async_runtime::spawn(run_connection(
//                 (self.ctx_fn)(&window),
//                 self.router.clone(),
//                 SinkAndStream::new(sink, rx.map(Ok)),
//                 Some(clear_subscriptions_rx),
//             ));
//         }

//         window.on_window_event(self.clone().on_window_event_handler(window.clone()))
//     }

//     #[allow(clippy::unwrap_used)] // TODO: Stop using unwrap
//     pub fn close_requested(&self, window: &Window<R>) {
//         let mut hasher = DefaultHasher::new();
//         window.hash(&mut hasher);
//         let window_hash = hasher.finish();

//         if let Some(shutdown_streams_tx) = self.windows.lock().unwrap().get(&window_hash) {
//             shutdown_streams_tx.unbounded_send(()).ok();
//         }
//     }

//     pub fn on_window_event_handler(
//         self: Arc<Self>,
//         window: Window<R>,
//     ) -> impl Fn(&WindowEvent) + Send + 'static {
//         move |event| {
//             if let WindowEvent::CloseRequested { .. } = event {
//                 self.close_requested(&window);
//             }
//         }
//     }
// }

// const PLUGIN_NAME: &str = "rspc";

// macro_rules! specta_builder {
//     () => {
//         tauri_specta::ts::builder().events(tauri_specta::collect_events![Msg, TransportResp])
//     };
// }

// #[cfg(test)]
// mod test {
//     use super::*;
//     use std::path::PathBuf;

//     #[test]
//     fn export_types() {
//         specta_builder!()
//             .path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./packages/tauri/src/types.ts"))
//             .export_for_plugin(PLUGIN_NAME)
//             .unwrap();
//     }
// }

// pub fn plugin<TCtx, R>(
//     router: Arc<Router2<TCtx>>,
//     ctx_fn: impl Fn(&Window<R>) -> TCtx + Send + Sync + 'static,
// ) -> TauriPlugin<R>
// where
//     TCtx: Clone + Send + Sync + 'static,
//     R: Runtime,
// {
//     let manager = WindowManager::new(ctx_fn, router);

//     let plugin_utils = specta_builder!().into_plugin_utils(PLUGIN_NAME);

//     Builder::<R, ()>::new(PLUGIN_NAME)
//         .invoke_handler(plugin_utils.invoke_handler)
//         .setup(|app| {
//             (plugin_utils.setup)(app);
//             Ok(())
//         })
//         .on_page_load(move |window, _page| {
//             manager.clone().on_page_load(window.clone());
//         })
//         .build()
// }
