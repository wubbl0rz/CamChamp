use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};
use std::future::Future;
use std::sync::{Arc, OnceLock, RwLock};
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use webrtc::api::{API, APIBuilder};
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264};
use webrtc::api::setting_engine::SettingEngine;
use webrtc::ice::network_type::NetworkType;
use webrtc::ice::udp_mux::{UDPMuxDefault, UDPMuxParams};
use webrtc::ice::udp_network::UDPNetwork;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType};
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use crate::registry::Registry;

mod registry;

static TRACKS: OnceLock<RwLock<HashMap<u32, Arc<TrackLocalStaticSample>>>> = OnceLock::new();
static RUNTIME: OnceLock<Runtime> = OnceLock::new();
static APPSTATE: OnceLock<AppState> = OnceLock::new();

struct AppState {
    api: API,
    connections: Registry<RTCPeerConnection>
}

impl AppState {
    // fn block_on<F: Future>(&self, future: F) -> F::Output {
    //     self.runtime.block_on(future)
    // }
}

pub fn get_capabilities() -> RTCRtpCodecCapability {
    return RTCRtpCodecCapability {
        mime_type: MIME_TYPE_H264.to_owned(),
        clock_rate: 90000,
        channels: 0,
        sdp_fmtp_line: "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f".to_owned(),
        rtcp_feedback: vec![],
    }
}

#[repr(C)]
pub struct ConnectionResult {
    client_id: u32,
    offer: *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn create_track() {
    RUNTIME.get().unwrap().block_on(create_track_internal())
}

pub async fn create_track_internal() {
    let track_id = 2;

    let track = TrackLocalStaticSample::new(get_capabilities(),
                                            track_id.to_string(),
                                            track_id.to_string());

    let mut tracks = TRACKS
        .get()
        .unwrap()
        .write()
        .unwrap();

    tracks.insert(track_id, Arc::new(track));
}

#[no_mangle]
pub unsafe extern "C" fn set_answer(id : u32, ptr : *const c_char) {
    RUNTIME.get().unwrap().block_on(set_answer_internal(id, ptr));
    println!("KEKL")
}

pub async unsafe fn set_answer_internal(id : u32, ptr : *const c_char) {
    let connections = &APPSTATE.get().unwrap().connections;

    let peer_connection = connections.get(id as u32);

    let sdp = CStr::from_ptr(ptr).to_str().unwrap().to_owned();

    let answer = RTCSessionDescription::answer(sdp).unwrap();

    peer_connection.set_remote_description(answer).await.unwrap();
}

#[no_mangle]
pub extern "C" fn create_connection() -> ConnectionResult {
    RUNTIME.get().unwrap().block_on(create_connection_internal())
}

pub async fn create_connection_internal() -> ConnectionResult {
    let api = &APPSTATE.get().unwrap().api;
    let tracks = TRACKS
        .get()
        .unwrap()
        .read()
        .unwrap();

    let peer_connection = api
        .new_peer_connection(RTCConfiguration::default())
        .await
        .unwrap();
    
    //peer_connection.on_peer_connection_state_change()

    for (_, track) in tracks.iter() {
        peer_connection.add_track(track.clone()).await.unwrap();
    }

    let offer = peer_connection.create_offer(None).await.unwrap();

    let mut gather_complete = peer_connection.gathering_complete_promise().await;
    peer_connection.set_local_description(offer).await.unwrap();
    let _ = gather_complete.recv().await;

    let connections = &APPSTATE.get().unwrap().connections;

    let str = CString::new(peer_connection.local_description()
        .await.unwrap().sdp)
        .unwrap();

    let id = connections.add(peer_connection);

    return ConnectionResult {
        client_id: id,
        offer: str.into_raw()
    }
}

#[no_mangle]
pub extern "C" fn init() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    RUNTIME.set(runtime).unwrap();

    RUNTIME.get().unwrap().block_on(init_internal())
}

pub async fn init_internal() {
    let mut media_engine = MediaEngine::default();

    media_engine.register_codec(RTCRtpCodecParameters{
        capability: get_capabilities(),
        payload_type: 96,
        ..Default::default()
    },RTPCodecType::Video).unwrap();

    // let udp_socket = UdpSocket::bind(("0.0.0.0", 36363)).await.unwrap();
    // let udp_mux = UDPMuxDefault::new(UDPMuxParams::new(udp_socket));

    //settings_engine.set_udp_network(UDPNetwork::Muxed(udp_mux));

    let mut setting_engine = SettingEngine::default();
    setting_engine.set_network_types(vec![NetworkType::Udp4]);
    setting_engine.set_udp_network(
        UDPNetwork::Muxed(
            UDPMuxDefault::new(
                UDPMuxParams::new(
                    tokio::net::UdpSocket::bind("0.0.0.0:36363").await.unwrap()
                )
            )
        )
    );

    let api = APIBuilder::new()
        .with_setting_engine(setting_engine)
        .with_media_engine(media_engine)
        .build();

    let _ = APPSTATE.set(AppState{
        api,
        connections: Registry::new()
    });
    let _ = TRACKS.set(RwLock::new(HashMap::new()));

    //TODO: cleanup old connections
}

fn main() {

}

