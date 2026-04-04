// backend/src/sfu.rs
// SFU (Selective Forwarding Unit) pour appels groupe 3+ participants.
// Base sur l'exemple rustrtc_sfu.rs — adapte pour signaling via WS Nook (pas DataChannel).
// Pattern: Room -> Peers -> MediaRelay -> RTCP PLI forwarding.

use rustrtc::{
    RtcConfiguration, RtpCodecParameters, SdpType, SessionDescription,
    media::{self, MediaKind, MediaStreamTrack},
    media::track::MediaRelay,
    peer_connection::{PeerConnection, PeerConnectionEvent},
    rtp::RtcpPacket,
    transports::ice::IceCandidate,
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::RwLock;
use tracing::{info, warn};

// ================================================================
// Structures
// ================================================================

/// Une track recue d un participant, relayee aux autres.
pub struct TrackInfo {
    pub relay: MediaRelay,
    pub remote_track: Arc<dyn MediaStreamTrack>,
    pub user_id: String,
    pub peer_id: u64,
    pub kind: MediaKind,
    pub params: RtpCodecParameters,
}

/// Un participant dans une room SFU.
pub struct Peer {
    pub id: u64,
    pub user_id: String,
    pub pc: PeerConnection,
    pub negotiation_pending: Arc<AtomicBool>,
    pub added_sources: RwLock<HashSet<String>>,
}

/// Une room SFU (liee a un conversation_id).
pub struct Room {
    pub room_id: String,
    pub peers: RwLock<HashMap<String, Arc<Peer>>>,
    pub tracks: RwLock<Vec<Arc<TrackInfo>>>,
}

/// Reponse SFU avec offre de renegotiation pending.
#[derive(Serialize, Debug)]
pub struct SfuJoinResponse {
    pub answer: String,
    pub peers: Vec<String>,
    pub renegotiate_offer: Option<String>,
}

/// Reponse de renegotiation du SFU vers le client.
#[derive(Serialize, Debug)]
pub struct SfuRenegotiateResponse {
    pub offer: String,
}

/// Etat global du SFU.
#[derive(Clone)]
pub struct SfuState {
    rooms: Arc<RwLock<HashMap<String, Arc<Room>>>>,
    next_peer_id: Arc<AtomicU64>,
}

// ================================================================
// Impl SfuState
// ================================================================

impl SfuState {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            next_peer_id: Arc::new(AtomicU64::new(1)),
        }
    }

    async fn get_or_create_room(&self, conversation_id: &str) -> Arc<Room> {
        let mut rooms = self.rooms.write().await;
        rooms
            .entry(conversation_id.to_string())
            .or_insert_with(|| Arc::new(Room {
                room_id: conversation_id.to_string(),
                peers: RwLock::new(HashMap::new()),
                tracks: RwLock::new(Vec::new()),
            }))
            .clone()
    }

    // ============================================================
    // SIGNALISATION
    // ============================================================

    /// Un participant rejoint une room avec une offre SDP.
    pub async fn handle_join(
        &self,
        user_id: &str,
        conversation_id: &str,
        offer_sdp: &str,
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

        // Creer PeerConnection
        let config = RtcConfiguration::default();
        let pc = PeerConnection::new(config.clone());

        let peer_id = self.next_peer_id.fetch_add(1, Ordering::Relaxed);
        let peer = Arc::new(Peer {
            id: peer_id,
            user_id: user_id.to_string(),
            pc: pc.clone(),
            negotiation_pending: Arc::new(AtomicBool::new(false)),
            added_sources: RwLock::new(HashSet::new()),
        });

        // Setup event handlers AVANT le SDP handshake
        Self::setup_peer_events(pc.clone(), room.clone(), peer.clone());

        // Inserer le peer
        {
            let mut peers = room.peers.write().await;
            peers.insert(user_id.to_string(), peer.clone());
        }

        // Parser et appliquer l offre distante
        let desc = SessionDescription::parse(SdpType::Offer, offer_sdp)
            .map_err(|e| format!("parse offer: {}", e))?;
        pc.set_remote_description(desc).await
            .map_err(|e| format!("set_remote_description: {}", e))?;

        // Attendre un candidate ICE pour peupler la reponse
        {
            let mut rx = pc.subscribe_ice_candidates();
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(3),
                rx.recv(),
            ).await;
        }

        // Creer answer
        let answer = pc.create_answer().await
            .map_err(|e| format!("create_answer: {}", e))?;
        pc.set_local_description(answer.clone())
            .map_err(|e| format!("set_local_description: {}", e))?;

        // Ajouter les tracks existantes de la room a ce nouveau peer
        Self::add_existing_tracks(peer.clone(), room.clone(), true).await;

        // Verifier si une renegotiation est pending
        let renegotiate_offer = if peer.negotiation_pending.load(Ordering::SeqCst) {
            peer.negotiation_pending.store(false, Ordering::SeqCst);
            pc.local_description().map(|d| d.to_sdp_string())
        } else {
            None
        };

        // Collecter les autres participants
        let others: Vec<String> = {
            let peers = room.peers.read().await;
            peers.keys().filter(|k| *k != user_id).cloned().collect()
        };

        info!(room=%conversation_id, peers=%others.len(), "SFU join complete for {}", user_id);

        Ok(SfuJoinResponse {
            answer: answer.to_sdp_string(),
            peers: others,
            renegotiate_offer,
        })
    }

    /// Le client repond a une offre de renegotiation du SFU.
    pub async fn handle_answer(
        &self,
        user_id: &str,
        conversation_id: &str,
        answer_sdp: &str,
    ) -> Result<(), String> {
        let rooms = self.rooms.read().await;
        let room = rooms.get(conversation_id)
            .ok_or_else(|| format!("room {} not found", conversation_id))?;
        let peers = room.peers.read().await;
        let peer = peers.get(user_id)
            .ok_or_else(|| format!("peer {} not found", user_id))?;

        let desc = SessionDescription::parse(SdpType::Answer, answer_sdp)
            .map_err(|e| format!("parse answer: {}", e))?;
        peer.pc.set_remote_description(desc).await
            .map_err(|e| format!("set_remote_answer: {}", e))?;

        info!(user=%user_id, "SFU renegotiation answer applied");
        Ok(())
    }

    /// Recois un ICE candidate du client.
    pub async fn handle_candidate(
        &self,
        user_id: &str,
        conversation_id: &str,
        candidate: &str,
    ) -> Result<(), String> {
        if candidate.is_empty() {
            return Ok(());
        }
        let rooms = self.rooms.read().await;
        let room = rooms.get(conversation_id)
            .ok_or_else(|| format!("room {} not found", conversation_id))?;
        let peers = room.peers.read().await;
        let peer = peers.get(user_id)
            .ok_or_else(|| format!("peer {} not found", user_id))?;

        match IceCandidate::from_sdp(candidate) {
            Ok(c) => peer.pc.add_ice_candidate(c)
                .map_err(|e| format!("add_ice_candidate: {}", e)),
            Err(e) => {
                warn!(user=%user_id, "SFU invalid ICE candidate: {}", e);
                Ok(())
            }
        }
    }

    /// Un participant quitte la room.
    pub async fn remove_peer(
        &self,
        user_id: &str,
        conversation_id: &str,
    ) -> Result<Vec<String>, String> {
        info!(user=%user_id, room=%conversation_id, "SFU remove_peer");
        let rooms = self.rooms.read().await;
        let room = rooms.get(conversation_id)
            .ok_or_else(|| format!("room {} not found", conversation_id))?;

        {
            let mut peers = room.peers.write().await;
            if let Some(p) = peers.remove(user_id) {
                p.pc.close();
            }
        }

        {
            let mut tracks = room.tracks.write().await;
            tracks.retain(|t| t.user_id != user_id);
        }

        {
            let peers = room.peers.read().await;
            for (_uid, peer) in peers.iter() {
                let mut added = peer.added_sources.write().await;
                added.retain(|k| !k.starts_with(&format!("{}:", user_id)));
            }
        }

        Ok({
            let peers = room.peers.read().await;
            peers.keys().cloned().collect()
        })
    }

    /// Recuperer l offre de renegotiation pending d un peer.
    pub async fn drain_pending_offer(&self, user_id: &str, conversation_id: &str) -> Option<SfuRenegotiateResponse> {
        let rooms = self.rooms.read().await;
        let room = rooms.get(conversation_id)?;
        let peers = room.peers.read().await;
        let peer = peers.get(user_id)?;

        if peer.negotiation_pending.load(Ordering::SeqCst) {
            peer.negotiation_pending.store(false, Ordering::SeqCst);
            if let Some(desc) = peer.pc.local_description() {
                info!(user=%user_id, "SFU pending offer drained");
                return Some(SfuRenegotiateResponse {
                    offer: desc.to_sdp_string(),
                });
            }
        }
        None
    }

    // ============================================================
    // EVENT HANDLING
    // ============================================================

    fn setup_peer_events(pc: PeerConnection, room: Arc<Room>, peer: Arc<Peer>) {
        let user_id = peer.user_id.clone();
        let peer_id = peer.id;
        let room_clone = room.clone();
        let __peer_clone = peer.clone();

        // Surveiller l etat ICE
        let mut ice_rx = pc.subscribe_ice_connection_state();
        let pc_for_ice = pc.clone();
        let uid_ice = user_id.clone();
        tokio::spawn(async move {
            while let Ok(()) = ice_rx.changed().await {
                let state = *ice_rx.borrow();
                info!(user=%uid_ice, state=?state, "SFU ICE connection state");
                match state {
                    rustrtc::IceConnectionState::Disconnected
                    | rustrtc::IceConnectionState::Failed
                    | rustrtc::IceConnectionState::Closed => {
                        info!(user=%uid_ice, "SFU closing PC on ICE disconnect");
                        pc_for_ice.close();
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Boucle d evenements
        tokio::spawn(async move {
            while let Some(event) = pc.recv().await {
                match event {
                    PeerConnectionEvent::Track(transceiver) => {
                        let receiver = match transceiver.receiver() {
                            Some(r) => r,
                            None => { warn!("SFU: no receiver on track event"); continue; }
                        };
                        let uid = user_id.clone();
                        let track = receiver.track();
                        let kind = track.kind();
                        info!(user=%uid, kind=?kind, "SFU track received from peer");

                        // Dedupliquer: une track par (user, kind)
                        {
                            let tracks = room_clone.tracks.read().await;
                            if tracks.iter().any(|t| t.user_id == uid && t.kind == kind) {
                                info!(user=%uid, kind=?kind, "SFU duplicate track, skipping");
                                continue;
                            }
                        }

                        // Creer un relay pour cette track
                        let (clock_rate, payload_type, channels) = if kind == MediaKind::Video {
                            (90_000u32, 96u8, 0u8)
                        } else {
                            (48_000u32, 111u8, 2u8)
                        };

                        let (source, local_track, _) = media::sample_track(kind, clock_rate as usize);
                        let relay = MediaRelay::with_capacity(local_track.clone(), 500);

                        let params = RtpCodecParameters {
                            payload_type,
                            clock_rate,
                            channels,
                        };

                        let track_info = Arc::new(TrackInfo {
                            relay,
                            remote_track: track.clone(),
                            user_id: user_id.clone(),
                            peer_id,
                            kind,
                            params: params.clone(),
                        });

                        // Ajouter aux tracks de la room
                        {
                            let mut tracks = room_clone.tracks.write().await;
                            tracks.push(track_info.clone());
                        }

                        // Ajouter cette track aux AUTRES peers
                        {
                            let peers = room_clone.peers.read().await;
                            for (other_id, other_peer) in peers.iter() {
                                if *other_id == uid { continue; }
                                let source_key = format!("{}:{}:{:?}", uid, peer_id, kind);
                                {
                                    let added = other_peer.added_sources.read().await;
                                    if added.contains(&source_key) {
                                        info!(to=%other_id, "SFU track already added, skip");
                                        continue;
                                    }
                                }

                                let relay_track = track_info.relay.subscribe();
                                match other_peer.pc.add_track_with_stream_id(
                                    relay_track,
                                    track_info.user_id.clone(),
                                    track_info.params.clone(),
                                ) {
                                    Ok(sender) => {
                                        {
                                            let mut added = other_peer.added_sources.write().await;
                                            added.insert(source_key.clone());
                                        }
                                        info!(from=%uid, to=%other_id, kind=?kind, "SFU track relayed to peer");

                                        // Forward PLI/RTCP de l autre peer vers la source
                                        let remote_track = track.clone();
                                        let mut rtcp_rx = sender.subscribe_rtcp();
                                        let other_log = other_id.clone();
                                        tokio::spawn(async move {
                                            while let Ok(packet) = rtcp_rx.recv().await {
                                                match packet {
                                                    RtcpPacket::PictureLossIndication(_)
                                                    | RtcpPacket::FullIntraRequest(_) => {
                                                        info!(from=%other_log, "SFU forwarding PLI to source");
                                                        let _ = remote_track.request_key_frame().await;
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        });

                                        // Trigger renegotiation pour ce peer
                                        Self::negotiate(other_peer.clone()).await;
                                    }
                                    Err(e) => {
                                        warn!(to=%other_id, "SFU add_track failed: {}", e);
                                    }
                                }
                            }
                        }

                        // Forward loop: track distante -> relay source
                        let incoming = track.clone();
                        tokio::spawn(async move {
                            while let Ok(mut sample) = incoming.recv().await {
                                match &mut sample {
                                    media::MediaSample::Video(f) => {
                                        f.header_extension = None;
                                        f.payload_type = None;
                                        if f.data.is_empty() { continue; }
                                    }
                                    media::MediaSample::Audio(f) => {
                                        f.payload_type = None;
                                        if f.data.is_empty() { continue; }
                                    }
                                }
                                if source.send(sample).await.is_err() { break; }
                            }
                            info!(user=%uid, kind=?kind, "SFU forward loop ended");
                        });

                        // PLI periodique toutes les 3 secondes
                        let pli_track = track.clone();
                        let uid_pli = uid.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
                            loop {
                                interval.tick().await;
                                if let Err(e) = pli_track.request_key_frame().await {
                                    warn!(user=%uid_pli, "SFU PLI failed: {}", e);
                                }
                            }
                        });
                    }

                    PeerConnectionEvent::DataChannel(_dc) => {
                        // Chat/text est deja gere par le WS Nook
                    }
                }
            }

            // === Peer disconnect cleanup ===
            info!(user=%user_id, "SFU peer event loop ended");
            {
                let mut tracks = room_clone.tracks.write().await;
                tracks.retain(|t| !(t.user_id == user_id && t.peer_id == peer_id));
            }
            {
                let mut peers = room_clone.peers.write().await;
                if let Some(current) = peers.get(&user_id) {
                    if current.id == peer_id {
                        peers.remove(&user_id);
                    }
                }
            }
            {
                for (_, other) in room_clone.peers.read().await.iter() {
                    let mut added = other.added_sources.write().await;
                    added.retain(|k| !k.starts_with(&format!("{}:{}:", user_id, peer_id)));
                }
            }
        });
    }

    // ============================================================
    // TRACKS EXISTANTES
    // ============================================================

    async fn add_existing_tracks(peer: Arc<Peer>, room: Arc<Room>, trigger_negotiation: bool) {
        let user_id = peer.user_id.clone();
        let tracks = room.tracks.read().await;
        let mut added_any = false;

        for track_info in tracks.iter() {
            if track_info.user_id == user_id { continue; }

            let source_key = format!("{}:{:?}:{:?}", track_info.user_id, track_info.peer_id, track_info.kind);
            {
                let added = peer.added_sources.read().await;
                if added.contains(&source_key) { continue; }
            }

            let relay_track = track_info.relay.subscribe();
            match peer.pc.add_track_with_stream_id(
                relay_track,
                track_info.user_id.clone(),
                track_info.params.clone(),
            ) {
                Ok(sender) => {
                    added_any = true;
                    {
                        let mut added = peer.added_sources.write().await;
                        added.insert(source_key);
                    }
                    info!(to=%user_id, from=%track_info.user_id, kind=?track_info.kind,
                          "SFU existing track added to new peer");

                    // Forward PLI
                    let remote_track = track_info.remote_track.clone();
                    let mut rtcp_rx = sender.subscribe_rtcp();
                    let uid = user_id.clone();
                    tokio::spawn(async move {
                        while let Ok(packet) = rtcp_rx.recv().await {
                            match packet {
                                RtcpPacket::PictureLossIndication(_)
                                | RtcpPacket::FullIntraRequest(_) => {
                                    info!(user=%uid, "SFU forwarding PLI (existing track)");
                                    let _ = remote_track.request_key_frame().await;
                                }
                                _ => {}
                            }
                        }
                    });
                }
                Err(e) => warn!(user=%user_id, "SFU add_existing_track failed: {}", e),
            }
        }

        if added_any && trigger_negotiation {
            Self::negotiate(peer).await;
        }
    }

    // ============================================================
    // NEGOTIATION (renomme `neg` pour eviter conflit avec rustrtc::negotiate)
    // ============================================================

    async fn negotiate(peer: Arc<Peer>) {
        use rustrtc::peer_connection::SignalingState;
        let signaling_state = peer.pc.signaling_state();

        if signaling_state != SignalingState::Stable {
            info!(user=%peer.user_id, state=?signaling_state, "SFU negotiation deferred");
            peer.negotiation_pending.store(true, Ordering::SeqCst);
            return;
        }

        peer.negotiation_pending.store(false, Ordering::SeqCst);

        match peer.pc.create_offer().await {
            Ok(offer) => {
                if let Err(e) = peer.pc.set_local_description(offer.clone()) {
                    warn!(user=%peer.user_id, "SFU set_local in negotiate failed: {}", e);
                    peer.negotiation_pending.store(true, Ordering::SeqCst);
                    return;
                }
                info!(user=%peer.user_id, "SFU renegotiation offer created");
                // L offre est maint dans peer.pc.local_description()
            }
            Err(e) => {
                warn!(user=%peer.user_id, "SFU create_offer in negotiate failed: {}", e);
                peer.negotiation_pending.store(true, Ordering::SeqCst);
            }
        }
    }
}