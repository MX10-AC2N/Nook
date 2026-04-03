// backend/src/sfu.rs
// SFU (Selective Forwarding Unit) pour appels groupe 3+ participants.
// Relais les flux media entre pairs via la crate rustrtc.

use std::collections::HashMap;
use std::sync::Arc;

use rustrtc::{
    RtcConfiguration,
    media::{MediaKind, MediaStreamTrack},
    media::track::MediaRelay,
    peer_connection::{PeerConnection, PeerConnectionEvent, PeerConnectionState},
    sdp::{Direction, RtpCodecParameters},
    SdpType, SessionDescription,
};
use rustrtc::transports::sctp::DataChannelState;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

// ================================================================
// Structures d'etat
// ================================================================

/// Track recue d'un participant, a relayer aux autres.
struct RelayTrack {
    user_id: String,
    track: Arc<dyn MediaStreamTrack>,
    kind: MediaKind,
}

/// Un participant dans une room SFU.
struct SfuPeer {
    user_id: String,
    pc: PeerConnection,
}

/// Une room SFU (liee a un conversation_id).
pub struct SfuRoom {
    pub room_id: String,
    pub peers: Arc<RwLock<HashMap<String, Arc<SfuPeer>>>>,
    pub tracks: Arc<RwLock<Vec<RelayTrack>>>,
}

/// Etat global du SFU.
#[derive(Clone)]
pub struct SfuState {
    rooms: Arc<RwLock<HashMap<String, Arc<SfuRoom>>>>,
}

// ================================================================
// DTOs pour la signalisation WS
// ================================================================

#[derive(Serialize, Debug)]
pub struct SfuJoinResponse {
    pub answer: String,
    pub peers: Vec<String>,
}

// ================================================================
// Impl SfuState
// ================================================================

impl SfuState {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Recupere ou cree une room SFU pour une conversation.
    async fn get_or_create_room(&self, conversation_id: &str) -> Arc<SfuRoom> {
        let mut rooms = self.rooms.write().await;
        rooms
            .entry(conversation_id.to_string())
            .or_insert_with(|| Arc::new(SfuRoom {
                room_id: conversation_id.to_string(),
                peers: Arc::new(RwLock::new(HashMap::new())),
                tracks: Arc::new(RwLock::new(Vec::new())),
            }))
            .clone()
    }

    /// Un participant rejoint une room avec une offre SDP.
    /// Retourne (answer_sdp, liste_autres_participants).
    pub async fn handle_join(
        &self,
        user_id: &str,
        conversation_id: &str,
        offer_sdp: String,
    ) -> Result<SfuJoinResponse, String> {
        info!(user=%user_id, room=%conversation_id, "SFU join request");
        let room = self.get_or_create_room(conversation_id).await;

        // Supprimer ancien peer si reconnect
        {
            let mut peers = room.peers.write().await;
            if let Some(old) = peers.remove(user_id) {
                warn!(user=%user_id, "SFU replacing old peer connection");
                old.pc.close();
            }
        }

        // Creer nouvelle PeerConnection avec les codecs
        let mut config = RtcConfiguration::default();
        config.audio_codecs.push(RtpCodecParameters {
            payload_type: 111,
            mime_type: "audio/opus".to_owned(),
            clock_rate: 48_000,
            channels: Some(2),
            fmtp: Some("minptime=10;useinbandfec=1".to_owned()),
            rtcp_feedback: vec!["transport-cc".to_owned()],
        });
        config.video_codecs = rustrtc::config::VideoCapability::defaults_vp8_av1();

        let pc = PeerConnection::new(&config);

        // Configurer events AVANT de traiter l'offre
        let pc_clone = pc.clone();
        let room_clone = room.clone();
        let uid_clone = user_id.to_string();
        Self::setup_events(pc_clone, room_clone, uid_clone);

        // Parser l'offre
        let offer_desc = SessionDescription::parse(&offer_sdp)
            .map_err(|e| format!("parse offer: {e}"))?;

        // Set remote description
        pc.set_remote_description(SdpType::Offer, &offer_desc)
            .await
            .map_err(|e| format!("set_remote_description: {e}"))?;

        // Creer answer
        let answer = pc.create_answer().await
            .map_err(|e| format!("create_answer: {e}"))?;
        pc.set_local_description(SdpType::Answer, &answer)
            .await
            .map_err(|e| format!("set_local_description: {e}"))?;

        // Inserer le peer
        let peer = Arc::new(SfuPeer {
            user_id: user_id.to_string(),
            pc: pc.clone(),
        });
        {
            let mut peers = room.peers.write().await;
            peers.insert(user_id.to_string(), peer);
        }

        // Ajouter les tracks existantes de la room a ce nouveau peer
        Self::add_existing_tracks(&room, user_id).await;

        // Collecter les autres participants
        let others = {
            let peers = room.peers.read().await;
            peers.keys().filter(|k| *k != user_id).cloned().collect::<Vec<_>>()
        };

        Ok(SfuJoinResponse {
            answer: answer.serialize(),
            peers: others,
        })
    }

    /// Ajoute un ICE candidate pour un participant.
    pub async fn handle_candidate(
        &self,
        user_id: &str,
        conversation_id: &str,
        candidate: String,
    ) -> Result<(), String> {
        let rooms = self.rooms.read().await;
        let room = rooms.get(conversation_id).ok_or_else(|| format!("Room {conversation_id} not found"))?;
        let peers = room.peers.read().await;
        let peer = peers.get(user_id).ok_or_else(|| format!("Peer {user_id} not found"))?;

        // Parser le candidate format SDP (a=candidate:...)
        let bare = candidate.trim_start_matches("a=").trim();
        match webrtc_sdp::attribute_type::SdpAttributeIceCandidate::from_string(bare) {
            Ok(c) => {
                peer.pc.add_ice_candidate(c).await
                    .map_err(|e| format!("add_ice_candidate: {e}"))
            }
            Err(e) => {
                warn!(user=%user_id, "SFU invalid ICE candidate: {e}");
                Ok(()) // non-fatal
            }
        }
    }

    /// Un participant quitte la room.
    pub async fn remove_peer(&self, user_id: &str, conversation_id: &str) -> Result<Vec<String>, String> {
        info!(user=%user_id, room=%conversation_id, "SFU remove_peer");
        let rooms = self.rooms.read().await;
        let room = rooms.get(conversation_id).ok_or_else(|| format!("Room {conversation_id} not found"))?;

        // Fermer et retirer le peer
        {
            let mut peers = room.peers.write().await;
            if let Some(peer) = peers.remove(user_id) {
                peer.pc.close();
            }
        }

        // Retirer les tracks de ce participant
        {
            let mut tracks = room.tracks.write().await;
            tracks.retain(|t| t.user_id != user_id);
        }

        let remaining = {
            let peers = room.peers.read().await;
            peers.keys().cloned().collect::<Vec<_>>()
        };
        Ok(remaining)
    }

    /// Configure les events d'une PeerConnection pour relayer les tracks.
    fn setup_events(pc: PeerConnection, room: Arc<SfuRoom>, user_id: String) {
        tokio::spawn(async move {
            let mut event_rx = pc.on_event().await;
            while let Some(event) = event_rx.recv().await {
                match event {
                    PeerConnectionEvent::TrackAdded(track) => {
                        info!(user=%user_id, kind=?track.kind().await, "SFU track received from peer");
                        // Stocker la track dans la room
                        room.tracks.write().await.push(RelayTrack {
                            user_id: user_id.clone(),
                            track: track.clone(),
                            kind: track.kind().await,
                        });

                        // Relayer a tous les AUTRES peers
                        let peers_guard = room.peers.read().await;
                        for (peer_uid, peer) in peers_guard.iter() {
                            if *peer_uid != user_id {
                                // Ajouter la track au PC du destinataire avec negociation
                                let params = Self::codec_params(&track.kind().await);
                                if let Err(e) = peer.pc.add_track(track.clone(), &params).await {
                                    error!(from=%user_id, to=%peer_uid, "SFU add_track failed: {e}");
                                }
                                // Renegocier: creer une nouvelle offre pour ce peer
                                if let Ok(new_offer) = peer.pc.create_offer().await {
                                    if peer.pc.set_local_description(SdpType::Offer, &new_offer).await.is_ok() {
                                        // On devrait envoyer new_offer.serialize() au client
                                        // via le WS handler — ce sera fait dans une V2
                                        info!(from=%user_id, to=%peer_uid, "SFU renegotiation offer created");
                                    }
                                }
                            }
                        }
                    }
                    PeerConnectionEvent::StateChange(state) => {
                        info!(user=%user_id, state=?state, "SFU PC state change");
                        if matches!(state, PeerConnectionState::Closed | PeerConnectionState::Failed) {
                            let _ = room.peers.write().await.remove(&user_id);
                            room.tracks.write().await.retain(|t| t.user_id != user_id);
                            info!(user=%user_id, "SFU peer cleaned up on disconnect");
                            return;
                        }
                    }
                    PeerConnectionEvent::NewIceCandidate(_candidate) => {
                        // Trickle ICE: on pourrait envoyer au client ici
                    }
                    PeerConnectionEvent::DataChannelEvent(_dc) => {
                        // Ignore pour le moment
                    }
                }
            }
        });
    }

    fn codec_params(kind: &MediaKind) -> RtpCodecParameters {
        match kind {
            MediaKind::Audio => RtpCodecParameters {
                payload_type: 111,
                mime_type: "audio/opus".to_owned(),
                clock_rate: 48_000,
                channels: Some(2),
                fmtp: Some("minptime=10;useinbandfec=1".to_owned()),
                rtcp_feedback: vec!["transport-cc".to_owned()],
            },
            MediaKind::Video => RtpCodecParameters {
                payload_type: 96,
                mime_type: "video/VP8".to_owned(),
                clock_rate: 90_000,
                channels: None,
                fmtp: None,
                rtcp_feedback: vec![
                    "goog-remb".to_owned(),
                    "transport-cc".to_owned(),
                    "ccm fir".to_owned(),
                    "nack".to_owned(),
                    "nack pli".to_owned(),
                ],
            },
            _ => RtpCodecParameters {
                payload_type: 0,
                mime_type: "audio/opus".to_owned(),
                clock_rate: 48_000,
                channels: Some(2),
                fmtp: None,
                rtcp_feedback: vec![],
            },
        }
    }

    /// Ajoute les tracks existantes de la room au nouveau peer.
    async fn add_existing_tracks(room: &Arc<SfuRoom>, user_id: &str) {
        let tracks = room.tracks.read().await;
        let peers = room.peers.read().await;
        if let Some(peer) = peers.get(user_id) {
            for rt in tracks.iter() {
                if rt.user_id != user_id {
                    let params = Self::codec_params(&rt.kind);
                    if let Err(e) = peer.pc.add_track(rt.track.clone(), &params).await {
                        warn!(user=%user_id, "SFU add_existing_track from {} failed: {e}", rt.user_id);
                    } else {
                        info!(user=%user_id, from=%rt.user_id, "SFU existing track added");
                    }
                }
            }
        }
    }
}

// ================================================================
// Helpers pour le WS handler (webrtc.rs)
// ================================================================

pub async fn handle_sfu_join_ws(
    sfu: &SfuState,
    user_id: &str,
    conversation_id: &str,
    sdp: String,
) -> Result<SfuJoinResponse, String> {
    sfu.handle_join(user_id, conversation_id, sdp).await
}

pub async fn handle_sfu_candidate_ws(
    sfu: &SfuState,
    user_id: &str,
    conversation_id: &str,
    candidate: String,
) -> Result<(), String> {
    sfu.handle_candidate(user_id, conversation_id, candidate).await
}

pub async fn handle_sfu_leave_ws(
    sfu: &SfuState,
    user_id: &str,
    conversation_id: &str,
) -> Result<Vec<String>, String> {
    sfu.remove_peer(user_id, conversation_id).await
}
