// backend/src/sfu.rs
// SFU (Selective Forwarding Unit) pour appels groupe 3+ participants
// Utilise la crate rustrtc pour relayer les medias entre pairs

use std::collections::HashMap;
use std::sync::Arc;

use rustrtc::{
    RtcConfiguration,
    media::{MediaKind, MediaStreamTrack},
    media::track::MediaRelay,
    peer_connection::{DisconnectReason, PeerConnection, PeerConnectionEvent, PeerConnectionState},
    SdpType, SessionDescription,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

// ================================================================
// Structures d'etat
// ================================================================

/// Track recu d'un participant, a relayer aux autres.
struct RelayTrack {
    user_id: String,
    track: Arc<dyn MediaStreamTrack>,
    kind: MediaKind,
}

/// Un participant dans une room SFU.
struct SfuPeer {
    user_id: String,
    pc: PeerConnection,
    /// Tracks locales envoyees par ce participant.
    local_tracks: RwLock<Vec<Arc<dyn MediaStreamTrack>>>,
}

/// Une room SFU (liee a un conversation_id).
pub struct SfuRoom {
    pub room_id: String,
    peers: RwLock<HashMap<String, Arc<SfuPeer>>>,
    tracks: RwLock<Vec<RelayTrack>>,
}

/// Etat global du SFU.
#[derive(Clone)]
pub struct SfuState {
    rooms: Arc<RwLock<HashMap<String, Arc<SfuRoom>>>>,
}

// ================================================================
// DTOs pour la signalisation
// ================================================================

#[derive(Deserialize, Debug)]
pub struct SfuJoinRequest {
    pub sdp: String,
    pub #[serde(rename = "type")]
    sdp_type: String,
    pub candidates: Option<Vec<String>>,
}

#[derive(Serialize, Debug)]
pub struct SfuJoinResponse {
    pub answer: String,
    pub peers: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct SfuCandidateRequest {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
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
                peers: RwLock::new(HashMap::new()),
                tracks: RwLock::new(Vec::new()),
            }))
            .clone()
    }

    /// Un participant rejoint une room avec une offre SDP.
    /// Retourne (answer_sdp, liste_autres_participants).
    pub async fn handle_join(
        &self,
        user_id: &str,
        conversation_id: &str,
        offer_sdp: &str,
    ) -> Result<SfuJoinResponse, String> {
        info!(user=%user_id, room=%conversation_id, "SFU join");
        let room = self.get_or_create_room(conversation_id).await;

        // Supprimer ancien peer si reconnect
        {
            let mut peers = room.peers.write().await;
            if let Some(old) = peers.remove(user_id) {
                warn!(user=%user_id, "SFU replacing old peer connection");
                old.pc.close();
            }
        }

        // Creer nouvelle PeerConnection
        let config = RtcConfiguration::default();
        let pc = PeerConnection::new(config);
        let pc_clone = pc.clone();
        let room_clone = room.clone();
        let uid_clone = user_id.to_string();

        // Configurer events AVANT set_remote_description
        Self::setup_events(pc_clone, room_clone, uid_clone.clone()).await;

        // Set remote offer
        pc.set_remote_description(SdpType::Offer, offer_sdp)
            .await
            .map_err(|e| format!("set_remote_description: {e}"))?;

        // Creer answer
        let answer = pc.create_answer()
            .await
            .map_err(|e| format!("create_answer: {e}"))?;
        pc.set_local_description(SdpType::Answer, &answer.sdp)
            .await
            .map_err(|e| format!("set_local_description: {e}"))?;

        // Inserer peer
        let peer = Arc::new(SfuPeer {
            user_id: user_id.to_string(),
            pc,
            local_tracks: RwLock::new(Vec::new()),
        });
        {
            let mut peers = room.peers.write().await;
            peers.insert(user_id.to_string(), peer);
        }

        // Ajouter les tracks existantes de la room a ce nouveau peer
        {
            let tracks = room.tracks.read().await;
            let p = room.peers.read().await;
            if let Some(new_peer) = p.get(user_id) {
                let tracks_clone = tracks.clone();
                drop(p);
                drop(tracks);
                Self::add_tracks_to_peer(&new_peer, tracks_clone.iter().collect()).await;
            }
        }

        // Collecter les autres participants
        let others = {
            let peers = room.peers.read().await;
            peers.keys()
                .filter(|k| *k != user_id)
                .cloned()
                .collect::<Vec<_>>()
        };

        Ok(SfuJoinResponse {
            answer: answer.sdp,
            peers: others,
        })
    }

    /// Ajoute un ICE candidate pour un participant.
    pub async fn handle_candidate(
        &self,
        user_id: &str,
        conversation_id: &str,
        candidate: &str,
    ) -> Result<(), String> {
        let rooms = self.rooms.read().await;
        let room = rooms.get(conversation_id)
            .ok_or_else(|| format!("Room {conversation_id} not found"))?;
        let peers = room.peers.read().await;
        let peer = peers.get(user_id)
            .ok_or_else(|| format!("Peer {user_id} not found"))?;

        // Parser le candidate format SDP (a=candidate:...)
        let bare = candidate.trim_start_matches("a=").trim();
        match webrtc_sdp::attribute_type::SdpAttributeIceCandidate::from_string(bare) {
            Ok(c) => {
                peer.pc.add_ice_candidate(c).await
                    .map_err(|e| format!("add_ice_candidate: {e}"))
            }
            Err(e) => {
                warn!(user=%user_id, "SFU invalid ICE candidate: {e}");
                // Non-fatal, on ignore
                Ok(())
            }
        }
    }

    /// Un participant quitte la room. Notifie les autres via leurs PeerConnections (event).
    pub async fn remove_peer(&self, user_id: &str, conversation_id: &str) -> Result<Vec<String>, String> {
        info!(user=%user_id, room=%conversation_id, "SFU leave");
        let rooms = self.rooms.read().await;
        let room = rooms.get(conversation_id)
            .ok_or_else(|| format!("Room {conversation_id} not found"))?;

        // Fermer la PC et retirer le peer
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

        // Renegocier pour les autres peers (enlever les tracks de cet utilisateur)
        // Note: on pourrait faire un renegotiation ici mais c'est optionnel — 
        // les tracks sont juste arretees, les autres continuent a jouer la derniere frame.

        // Retourner la liste des participants restants
        let remaining = {
            let peers = room.peers.read().await;
            peers.keys().cloned().collect::<Vec<_>>()
        };

        Ok(remaining)
    }

    /// Configure les events d'une PeerConnection pour relayer les tracks.
    async fn setup_events(pc: PeerConnection, room: Arc<SfuRoom>, user_id: String) {
        let receiver = pc.on_event().await;
        let room_clone = room.clone();
        let uid_clone = user_id.clone();
        
        tokio::spawn(async move {
            let mut recv = receiver;
            while let Some(event) = recv.recv().await {
                match event {
                    PeerConnectionEvent::TrackAdded(track) => {
                        // On recoit une track du participant. On la relaye aux autres.
                        let kind = match track.kind().await {
                            MediaKind::Audio => MediaKind::Audio,
                            MediaKind::Video => MediaKind::Video,
                        };
                        info!(user=%uid_clone, kind=?kind, "SFU track received");

                        // Ajouter aux tracks de la room
                        {
                            let mut tracks = room_clone.tracks.write().await;
                            tracks.push(RelayTrack {
                                user_id: uid_clone.clone(),
                                track: track.clone(),
                                kind,
                            });
                        }

                        // Relayer a tous les autres peers
                        let peers = room_clone.peers.read().await;
                        let (track, kind) = (track, kind);
                        for (peer_uid, peer) in peers.iter() {
                            if *peer_uid != uid_clone {
                                // Cloner la track et l'ajouter au PC du destinataire
                                match Self::add_remote_track(peer, &track, &kind).await {
                                    Ok(_) => {
                                        info!(from=%uid_clone, to=%peer_uid, "SFU track relayed");
                                    }
                                    Err(e) => {
                                        error!(from=%uid_clone, to=%peer_uid, "SFU relay error: {e}");
                                    }
                                }
                            }
                        }
                    }
                    PeerConnectionEvent::StateChange(state) => {
                        info!(user=%uid_clone, state=?state, "SFU PC state change");
                        if state == PeerConnectionState::Closed
                            || state == PeerConnectionState::Failed
                            || state == PeerConnectionState::Disconnected
                        {
                            // Le client s'est deconnecte, on nettoie
                            let _ = room_clone.peers.write().await.remove(&uid_clone);
                            let _ = room_clone.tracks.write().await
                                .drain(..)
                                .filter(|t| t.user_id != uid_clone)
                                .collect::<Vec<_>>();
                            info!(user=%uid_clone, "SFU peer cleaned up on disconnect");
                            return;
                        }
                    }
                    PeerConnectionEvent::IceCandidate(c) => {
                        // Envoyer ICE candidate au client via WebSocket
                        // Note: en mode SFU, le client recoit l'ICE via la reponse SDP,
                        // mais on peut aussi envoyer des trickle ICE ici
                        info!(user=%uid_clone, "SFU ICE candidate generated");
                    }
                    PeerConnectionEvent::Disconnected { reason: _, user: _ } 
                    | PeerConnectionEvent::DataChannelEvent(_) => {
                        // DataChannels: on les ignore pour le moment (chat/text deja gere par WS)
                    }
                }
            }
        });
    }

    /// Ajoute une track distante a la PC d'un autre participant.
    async fn add_remote_track(
        peer: &SfuPeer,
        track: &Arc<dyn MediaStreamTrack>,
        kind: &MediaKind,
    ) -> Result<(), String> {
        // Rattacher la track via le relay
        let params = match kind {
            MediaKind::Audio => rustrtc::RtpCodecParameters {
                payload_type: 111,
                mime_type: "audio/opus".to_string(),
                clock_rate: 48_000,
                channels: Some(2),
                rtcp_feedback: vec![],
                fmtp: Some("minptime=10;useinbandfec=1".to_string()),
            },
            MediaKind::Video => rustrtc::RtpCodecParameters {
                payload_type: 96,
                mime_type: "video/VP8".to_string(),
                clock_rate: 90_000,
                channels: None,
                rtcp_feedback: vec![
                    "goog-remb".to_string(),
                    "transport-cc".to_string(),
                    "ccm fir".to_string(),
                    "nack".to_string(),
                    "nack pli".to_string(),
                ],
                fmtp: None,
            },
        };
        peer.pc.add_track(track.clone(), &params).await
            .map_err(|e| format!("add_track: {e}"))
    }

    /// Ajoute les tracks existantes d'une room a un nouveau peer.
    async fn add_tracks_to_peer(peer: &SfuPeer, tracks: Vec<&RelayTrack>) {
        for rt in tracks {
            if rt.user_id != peer.user_id {
                if let Err(e) = Self::add_remote_track(peer, &rt.track, &rt.kind).await {
                    warn!(user=%peer.user_id, "SFU failed to add track from {}: {e}", rt.user_id);
                }
            }
        }
    }
}

// ================================================================
// Integration avec le WS handler (webrtc.rs)
// ================================================================

/// Appelle depuis le WS handler pour traiter `sfu_join`.
pub async fn handle_sfu_join_ws(
    sfu: &SfuState,
    user_id: &str,
    conversation_id: &str,
    sdp: &str,
    _broadcast_tx: &tokio::sync::broadcast::Sender<String>,
) -> Option<String> {
    match sfu.handle_join(user_id, conversation_id, sdp).await {
        Ok(resp) => {
            Ok(resp) // serialized and sent back by the WS handler
        }
        Err(e) => {
            error!(user=%user_id, "SFU join error: {e}");
            Err(e)
        }
    }
}

/// Appel depuis le WS handler pour traiter `sfu_candidate`.
pub async fn handle_sfu_candidate_ws(
    sfu: &SfuState,
    user_id: &str,
    conversation_id: &str,
    candidate: &str,
) -> Result<(), String> {
    sfu.handle_candidate(user_id, conversation_id, candidate).await
}

/// Appel depuis le WS handler pour traiter `sfu_leave`.
pub async fn handle_sfu_leave_ws(
    sfu: &SfuState,
    user_id: &str,
    conversation_id: &str,
) -> Result<Vec<String>, String> {
    sfu.remove_peer(user_id, conversation_id).await
}
