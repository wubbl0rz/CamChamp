use std::ffi::{c_char, CStr, CString};
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use tokio::net::UdpSocket;
use webrtc::api::{API, APIBuilder};
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264};
use webrtc::api::setting_engine::SettingEngine;
use webrtc::ice::network_type::NetworkType;
use webrtc::ice::udp_mux::{UDPMuxDefault, UDPMuxParams};
use webrtc::ice::udp_network::UDPNetwork;
use webrtc::media::Sample;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType};
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;

use state::AppState;

mod registry;
mod state;

pub fn get_capabilities() -> RTCRtpCodecCapability {
    return RTCRtpCodecCapability {
        mime_type: MIME_TYPE_H264.to_owned(),
        clock_rate: 90000,
        channels: 0,
        sdp_fmtp_line: "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f".to_owned(),
        rtcp_feedback: vec![],
    };
}

#[repr(C)]
pub struct ConnectionResult {
    client_id: u32,
    offer: *mut c_char,
}

#[no_mangle]
pub unsafe extern "C" fn send_frame(id: u32,  duration : f64, len: usize, ptr: *const u8) {
    AppState::INSTANCE().block_on(send_frame_internal(id, duration, len, ptr))
}

pub async fn send_frame_internal(id: u32, duration : f64, len: usize, ptr: *const u8) {
    let tracks = &AppState::INSTANCE().tracks;

    if let Some(track) = tracks.get(id) {
        let bytes = Bytes::from(unsafe {
            std::slice::from_raw_parts(ptr, len)
        });

        let micros = duration * 1000.0;

        let sample = Sample {
            data: bytes,
            duration: Duration::from_micros(micros as u64),
            ..Sample::default()
        };

        track.write_sample(&sample).await.unwrap();
    }
}

#[no_mangle]
pub unsafe extern "C" fn create_track() -> u32 {
    AppState::INSTANCE().block_on(create_track_internal())
}

pub async fn create_track_internal() -> u32 {
    let tracks = &AppState::INSTANCE().tracks;

    let (id, _) = tracks.add(|id| {
        let track = TrackLocalStaticSample::new(get_capabilities(),
                                                id.to_string(),
                                                id.to_string());
        track
    });

    return id;
}

#[no_mangle]
pub unsafe extern "C" fn set_answer(id: u32, ptr: *const c_char) {
    AppState::INSTANCE().block_on(set_answer_internal(id, ptr));
}

pub async unsafe fn set_answer_internal(id: u32, ptr: *const c_char) {
    let connections = &AppState::INSTANCE().connections;

    if let Some(peer_connection) = connections.get(id) {
        let sdp = CStr::from_ptr(ptr).to_str().unwrap().to_owned();

        let answer = RTCSessionDescription::answer(sdp).unwrap();

        peer_connection.set_remote_description(answer).await.unwrap();
    }
}

#[no_mangle]
pub extern "C" fn create_connection() -> ConnectionResult {
    AppState::INSTANCE().block_on(create_connection_internal())
}

pub async fn setup_connection() -> (u32, Arc<RTCPeerConnection>) {
    let api = &AppState::INSTANCE().api;
    let connections = &AppState::INSTANCE().connections;

    let peer_connection = api
        .new_peer_connection(RTCConfiguration::default())
        .await
        .unwrap();

    let (id, con) = connections.add(|id| {
        peer_connection.on_peer_connection_state_change(Box::new(move |s| {
            let connections = &AppState::INSTANCE().connections;

            if let Some(peer_connection) = connections.get(id) {
                match peer_connection.connection_state() {
                    RTCPeerConnectionState::Disconnected |
                    RTCPeerConnectionState::Failed |
                    RTCPeerConnectionState::Closed => {
                        connections.del(id);
                    }
                    _ => {}
                };

                println!("Peer Connection {id} State has changed: {s}");
            }

            Box::pin(async {})
        }));
        peer_connection
    });

    // clean abandoned connections
    tokio::spawn(async move {
        let connections = &AppState::INSTANCE().connections;

        tokio::time::sleep(Duration::from_secs(10)).await;

        if let Some(peer_connection) = connections.get(id) {
            if peer_connection.connection_state() == RTCPeerConnectionState::New {
                connections.del(id);
            }
        }
    });

    return (id, con);
}

pub async fn create_connection_internal() -> ConnectionResult {
    let tracks = &AppState::INSTANCE().tracks;

    let (id, peer_connection) = setup_connection().await;

    for kv in tracks.iter() {
        peer_connection.add_track(kv.value().clone()).await.unwrap();
    }

    let offer = peer_connection.create_offer(None).await.unwrap();

    let mut gather_complete = peer_connection.gathering_complete_promise().await;
    peer_connection.set_local_description(offer).await.unwrap();
    let _ = gather_complete.recv().await;

    let str = CString::new(peer_connection.local_description()
        .await.unwrap().sdp)
        .unwrap();

    return ConnectionResult {
        client_id: id,
        offer: str.into_raw(),
    };
}

#[no_mangle]
pub extern "C" fn init() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let api = runtime.block_on(init_internal());

    AppState::INIT(runtime, api, true);

    AppState::INSTANCE().block_on(async {
        tokio::spawn(async {
            let connections = &AppState::INSTANCE().connections;

            loop {
                tokio::time::sleep(Duration::from_secs(15)).await;

                let count = connections.len();

                println!("Active connections: {count}");
                println!("Active connections: {count}");
            }
        });
    });
}

async fn init_internal() -> API {
    let mut media_engine = MediaEngine::default();

    media_engine.register_codec(RTCRtpCodecParameters {
        capability: get_capabilities(),
        payload_type: 96,
        ..Default::default()
    }, RTPCodecType::Video).unwrap();

    let udp_socket = UdpSocket::bind(("0.0.0.0", 36363)).await.unwrap();
    let udp_mux = UDPMuxDefault::new(UDPMuxParams::new(udp_socket));

    let mut setting_engine = SettingEngine::default();
    setting_engine.set_network_types(vec![NetworkType::Udp4]);
    setting_engine.set_udp_network(UDPNetwork::Muxed(udp_mux));

    let api = APIBuilder::new()
        .with_setting_engine(setting_engine)
        .with_media_engine(media_engine)
        .build();

    return api;
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_char) {
    let _ = CString::from_raw(ptr);
}