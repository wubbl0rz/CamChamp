use std::future::Future;
use std::sync::OnceLock;

use tokio::runtime::Runtime;
use webrtc::api::API;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;

use crate::registry::Registry;

static APPSTATE: OnceLock<AppState> = OnceLock::new();

pub struct AppState {
    pub debug : bool,
    pub api: API,
    pub runtime: Runtime,
    pub connections: Registry<RTCPeerConnection>,
    pub tracks: Registry<TrackLocalStaticSample>
}

impl AppState {
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    #[allow(non_snake_case)]
    pub fn INIT(runtime : Runtime, api: API, debug : bool) {
        let _ = APPSTATE.set(AppState {
            debug,
            runtime,
            api,
            connections: Registry::new(),
            tracks: Registry::new(),
        });
    }

    #[allow(non_snake_case)]
    pub fn INSTANCE() -> &'static AppState {
        return APPSTATE.get().unwrap();
    }
}
