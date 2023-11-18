use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use webrtc::api::{API, APIBuilder};
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264};
use webrtc::api::setting_engine::SettingEngine;
use webrtc::ice::udp_mux::{UDPMuxDefault, UDPMuxParams};
use webrtc::ice::udp_network::UDPNetwork;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType};
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use uuid::{uuid, Uuid};

static API: OnceLock<API> = OnceLock::new();
static TRACKS: OnceLock<RwLock<HashMap<u128, Arc<TrackLocalStaticSample>>>> = OnceLock::new();
static CONNECTIONS: OnceLock<RwLock<HashMap<u128, Arc<RTCPeerConnection>>>> = OnceLock::new();
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

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
    client_id: u128,
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
pub unsafe extern "C" fn set_answer(id : u128, ptr : *const c_char) {
    RUNTIME.get().unwrap().block_on(set_answer_internal(id, ptr))
}

pub async unsafe fn set_answer_internal(id : u128, ptr : *const c_char) {
    let connections = CONNECTIONS
        .get()
        .unwrap()
        .read()
        .unwrap();

    let peer_connection = connections.get(&id).unwrap();

    let sdp = CStr::from_ptr(ptr).to_str().unwrap().to_owned();

    let answer = RTCSessionDescription::answer(sdp).unwrap();

    peer_connection.set_remote_description(answer).await.unwrap();
}

#[no_mangle]
pub extern "C" fn create_connection() -> ConnectionResult {
    RUNTIME.get().unwrap().block_on(create_connection_internal())
}

pub async fn create_connection_internal() -> ConnectionResult {
    let api = API.get().unwrap();
    let tracks = TRACKS
        .get()
        .unwrap()
        .read()
        .unwrap();

    let id = Uuid::new_v4().as_u128();

    dbg!(id);

    let peer_connection = api
        .new_peer_connection(RTCConfiguration::default())
        .await
        .unwrap();

    for (_, track) in tracks.iter() {
        peer_connection.add_track(track.clone()).await.unwrap();
    }

    let offer = peer_connection.create_offer(None).await.unwrap();

    let mut gather_complete = peer_connection.gathering_complete_promise().await;
    peer_connection.set_local_description(offer).await.unwrap();
    let _ = gather_complete.recv().await;

    let mut connections = CONNECTIONS
        .get()
        .unwrap()
        .write()
        .unwrap();

    let str = CString::new(peer_connection.local_description()
        .await.unwrap().sdp)
        .unwrap();

    connections.insert(id, Arc::new(peer_connection));

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

    let udp_socket = UdpSocket::bind(("0.0.0.0", 36363)).await.unwrap();
    let udp_mux = UDPMuxDefault::new(UDPMuxParams::new(udp_socket));

    let mut settings_engine = SettingEngine::default();
    settings_engine.set_udp_network(UDPNetwork::Muxed(udp_mux));

    let api = APIBuilder::new()
        .with_setting_engine(settings_engine)
        .with_media_engine(media_engine)
        .build();

    let _ = API.set(api);
    let _ = TRACKS.set(RwLock::new(HashMap::new()));
    let _ = CONNECTIONS.set(RwLock::new(HashMap::new()));
}

fn main() {

}

