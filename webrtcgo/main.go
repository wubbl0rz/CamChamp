package main

/*
#include "stdint.h"
#include "stdlib.h"
#cgo LDFLAGS: -Wl,--allow-multiple-definition

struct ConnectionResult {
    int clientId;
	char* offer;
};
*/
import "C"

import (
	"github.com/pion/ice/v3"
	"github.com/pion/webrtc/v4"
	"go.uber.org/atomic"
	"net"
	"strconv"
	"unsafe"
)

var TRACKS = make(map[int64]*webrtc.TrackLocalStaticSample)
var CONNECTIONS = make(map[int64]*webrtc.PeerConnection)
var api *webrtc.API
var trackCounter atomic.Int64
var connectionCounter atomic.Int64

var capability = webrtc.RTPCodecCapability{
	MimeType:     "video/H264",
	ClockRate:    90000,
	Channels:     0,
	SDPFmtpLine:  "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f",
	RTCPFeedback: nil,
}

//export setAnswer
func setAnswer(id C.int, answer_ptr *C.char) {
	pc := CONNECTIONS[int64(id)]
	var answer = C.GoString(answer_ptr)

	err := pc.SetRemoteDescription(webrtc.SessionDescription{
		Type: webrtc.SDPTypeAnswer,
		SDP:  answer,
	})

	if err != nil {
		return
	}
}

//export createConnection
func createConnection() C.struct_ConnectionResult {
	pc, _ := api.NewPeerConnection(webrtc.Configuration{})

	for _, track := range TRACKS {
		_, _ = pc.AddTrack(track)
	}

	offer, _ := pc.CreateOffer(nil)

	gatherComplete := webrtc.GatheringCompletePromise(pc)
	_ = pc.SetLocalDescription(offer)
	<-gatherComplete

	id := connectionCounter.Inc()

	CONNECTIONS[id] = pc

	return C.struct_ConnectionResult{
		offer:    C.CString(pc.LocalDescription().SDP),
		clientId: C.int(id),
	}
}

//export createTrack
func createTrack() C.int64_t {
	var trackId = trackCounter.Load()
	trackCounter.Inc()
	var trackIdStr = strconv.FormatInt(trackId, 10)

	track, err := webrtc.NewTrackLocalStaticSample(capability, trackIdStr, trackIdStr)

	if err != nil {
		panic(err)
	}

	TRACKS[trackId] = track

	return C.int64_t(trackId)
}

//export freeStr
func freeStr(ptr *C.char) {
	C.free(unsafe.Pointer(ptr))
}

func UseTCP() webrtc.SettingEngine {
	settingEngine := webrtc.SettingEngine{}

	settingEngine.SetNetworkTypes([]webrtc.NetworkType{
		webrtc.NetworkTypeTCP4,
	})

	tcpListener, err := net.ListenTCP("tcp", &net.TCPAddr{
		IP:   net.IP{0, 0, 0, 0},
		Port: 35353,
	})

	if err != nil {
		panic(err)
	}

	tcpMux := webrtc.NewICETCPMux(nil, tcpListener, 8)
	settingEngine.SetICETCPMux(tcpMux)

	return settingEngine
}

func UseUDP() webrtc.SettingEngine {
	settingEngine := webrtc.SettingEngine{}

	settingEngine.SetNetworkTypes([]webrtc.NetworkType{
		webrtc.NetworkTypeUDP4,
	})

	mux, err := ice.NewMultiUDPMuxFromPort(36363)

	if err != nil {
		panic(err)
	}

	settingEngine.SetICEUDPMux(mux)

	return settingEngine
}

//export initNetwork
func initNetwork() {
	m := &webrtc.MediaEngine{}

	err := m.RegisterCodec(webrtc.RTPCodecParameters{
		RTPCodecCapability: capability,
		PayloadType:        96,
	}, webrtc.RTPCodecTypeVideo)

	if err != nil {
		return
	}

	settingEngine := UseUDP()

	api = webrtc.NewAPI(webrtc.WithSettingEngine(settingEngine), webrtc.WithMediaEngine(m))
}

func main() {
}
