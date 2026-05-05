     1|// backend/src/sfu.rs
     2|// SFU (Selective Forwarding Unit) pour appels groupe 3+ participants.
     3|// Base sur l'exemple rustrtc_sfu.rs — adapte pour signaling via WS Nook (pas DataChannel).
     4|// Pattern: Room -> Peers -> MediaRelay -> RTCP PLI forwarding.
     5|
     6|#![allow(clippy::for_kv_map)]
     7|
     8|use rustrtc::{
     9|    RtcConfiguration, RtpCodecParameters, SdpType, SessionDescription,
    10|    media::{self, MediaKind, MediaStreamTrack},
    11|    media::track::MediaRelay,
    12|    peer_connection::{PeerConnection, PeerConnectionEvent},
    13|    rtp::RtcpPacket,
    14|    transports::ice::IceCandidate,
    15|};
    16|use serde::Serialize;
    17|use std::collections::{HashMap, HashSet};
    18|use std::sync::Arc;
    19|use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    20|use tokio::sync::RwLock;
    21|use tracing::{info, warn};
    22|
    23|// ================================================================
    24|// Structures
    25|// ================================================================
    26|
    27|/// Une track recue d un participant, relayee aux autres.
    28|pub struct TrackInfo {
    29|    pub relay: MediaRelay,
    30|    pub remote_track: Arc<dyn MediaStreamTrack>,
    31|    pub user_id: String,
    32|    pub peer_id: u64,
    33|    pub kind: MediaKind,
    34|    pub params: RtpCodecParameters,
    35|}
    36|
    37|/// Un participant dans une room SFU.
    38|pub struct Peer {
    39|    pub id: u64,
    40|    pub user_id: String,
    41|    pub pc: PeerConnection,
    42|    pub negotiation_pending: Arc<AtomicBool>,
    43|    pub added_sources: RwLock<HashSet<String>>,
    44|}
    45|
    46|/// Une room SFU (liee a un conversation_id).
    47|pub struct Room {
    48|    pub _room_id: String,
    49|    pub peers: RwLock<HashMap<String, Arc<Peer>>>,
    50|    pub tracks: RwLock<Vec<Arc<TrackInfo>>>,
    51|}
    52|
    53|/// Reponse SFU avec offre de renegotiation pending.
    54|#[derive(Serialize, Debug)]
    55|pub struct SfuJoinResponse {
    56|    pub answer: String,
    57|    pub peers: Vec<String>,
    58|    pub renegotiate_offer: Option<String>,
    59|}
    60|
    61|/// Reponse de renegotiation du SFU vers le client.
    62|#[derive(Serialize, Debug)]
    63|pub struct SfuRenegotiateResponse {
    64|    pub offer: String,
    65|}
    66|
    67|/// Etat global du SFU.
    68|#[derive(Clone)]
    69|pub struct SfuState {
    70|    rooms: Arc<RwLock<HashMap<String, Arc<Room>>>>,
    71|    next_peer_id: Arc<AtomicU64>,
    72|}
    73|
    74|// ================================================================
    75|// Impl SfuState
    76|// ================================================================
    77|
    78|impl SfuState {
    79|    pub fn new() -> Self {
    80|        Self {
    81|            rooms: Arc::new(RwLock::new(HashMap::new())),
    82|            next_peer_id: Arc::new(AtomicU64::new(1)),
    83|        }
    84|    }
    85|
    86|    async fn get_or_create_room(&self, conversation_id: &str) -> Arc<Room> {
    87|        let mut rooms = self.rooms.write().await;
    88|        rooms
    89|            .entry(conversation_id.to_string())
    90|            .or_insert_with(|| Arc::new(Room {
    91|                _room_id: conversation_id.to_string(),
    92|                peers: RwLock::new(HashMap::new()),
    93|                tracks: RwLock::new(Vec::new()),
    94|            }))
    95|            .clone()
    96|    }
    97|
    98|    // ============================================================
    99|    // SIGNALISATION
   100|    // ============================================================
   101|
   102|    /// Un participant rejoint une room avec une offre SDP.
   103|    pub async fn handle_join(
   104|        &self,
   105|        user_id: &str,
   106|        conversation_id: &str,
   107|        offer_sdp: &str,
   108|    ) -> Result<SfuJoinResponse, String> {
   109|        info!(user=%user_id, room=%conversation_id, "SFU join request");
   110|        let room = self.get_or_create_room(conversation_id).await;
   111|
   112|        // Supprimer ancien peer si reconnect
   113|        {
   114|            let mut peers = room.peers.write().await;
   115|            if let Some(old) = peers.remove(user_id) {
   116|                warn!(user=%user_id, "SFU replacing old peer connection");
   117|                old.pc.close();
   118|            }
   119|        }
   120|
   121|        // Creer PeerConnection
   122|        let config = RtcConfiguration::default();
   123|        let pc = PeerConnection::new(config.clone());
   124|
   125|        let peer_id = self.next_peer_id.fetch_add(1, Ordering::Relaxed);
   126|        let peer = Arc::new(Peer {
   127|            id: peer_id,
   128|            user_id: user_id.to_string(),
   129|            pc: pc.clone(),
   130|            negotiation_pending: Arc::new(AtomicBool::new(false)),
   131|            added_sources: RwLock::new(HashSet::new()),
   132|        });
   133|
   134|        // Setup event handlers AVANT le SDP handshake
   135|        Self::setup_peer_events(pc.clone(), room.clone(), peer.clone());
   136|
   137|        // Inserer le peer
   138|        {
   139|            let mut peers = room.peers.write().await;
   140|            peers.insert(user_id.to_string(), peer.clone());
   141|        }
   142|
   143|        // Parser et appliquer l offre distante
   144|        let desc = SessionDescription::parse(SdpType::Offer, offer_sdp)
   145|            .map_err(|e| format!("parse offer: {}", e))?;
   146|        pc.set_remote_description(desc).await
   147|            .map_err(|e| format!("set_remote_description: {}", e))?;
   148|
   149|        // Attendre un candidate ICE pour peupler la reponse
   150|        {
   151|            let mut rx = pc.subscribe_ice_candidates();
   152|            let _ = tokio::time::timeout(
   153|                std::time::Duration::from_secs(3),
   154|                rx.recv(),
   155|            ).await;
   156|        }
   157|
   158|        // Creer answer
   159|        let answer = pc.create_answer().await
   160|            .map_err(|e| format!("create_answer: {}", e))?;
   161|        pc.set_local_description(answer.clone())
   162|            .map_err(|e| format!("set_local_description: {}", e))?;
   163|
   164|        // Ajouter les tracks existantes de la room a ce nouveau peer
   165|        Self::add_existing_tracks(peer.clone(), room.clone(), true).await;
   166|
   167|        // Verifier si une renegotiation est pending
   168|        let renegotiate_offer = if peer.negotiation_pending.load(Ordering::SeqCst) {
   169|            peer.negotiation_pending.store(false, Ordering::SeqCst);
   170|            pc.local_description().map(|d| d.to_sdp_string())
   171|        } else {
   172|            None
   173|        };
   174|
   175|        // Collecter les autres participants
   176|        let others: Vec<String> = {
   177|            let peers = room.peers.read().await;
   178|            peers.keys().filter(|k| *k != user_id).cloned().collect()
   179|        };
   180|
   181|        info!(room=%conversation_id, peers=%others.len(), "SFU join complete for {}", user_id);
   182|
   183|        Ok(SfuJoinResponse {
   184|            answer: answer.to_sdp_string(),
   185|            peers: others,
   186|            renegotiate_offer,
   187|        })
   188|    }
   189|
   190|    /// Le client repond a une offre de renegotiation du SFU.
   191|    pub async fn handle_answer(
   192|        &self,
   193|        user_id: &str,
   194|        conversation_id: &str,
   195|        answer_sdp: &str,
   196|    ) -> Result<(), String> {
   197|        let rooms = self.rooms.read().await;
   198|        let room = rooms.get(conversation_id)
   199|            .ok_or_else(|| format!("room {} not found", conversation_id))?;
   200|        let peers = room.peers.read().await;
   201|        let peer = peers.get(user_id)
   202|            .ok_or_else(|| format!("peer {} not found", user_id))?;
   203|
   204|        let desc = SessionDescription::parse(SdpType::Answer, answer_sdp)
   205|            .map_err(|e| format!("parse answer: {}", e))?;
   206|        peer.pc.set_remote_description(desc).await
   207|            .map_err(|e| format!("set_remote_answer: {}", e))?;
   208|
   209|        info!(user=%user_id, "SFU renegotiation answer applied");
   210|        Ok(())
   211|    }
   212|
   213|    /// Recois un ICE candidate du client.
   214|    pub async fn handle_candidate(
   215|        &self,
   216|        user_id: &str,
   217|        conversation_id: &str,
   218|        candidate: &str,
   219|    ) -> Result<(), String> {
   220|        if candidate.is_empty() {
   221|            return Ok(());
   222|        }
   223|        let rooms = self.rooms.read().await;
   224|        let room = rooms.get(conversation_id)
   225|            .ok_or_else(|| format!("room {} not found", conversation_id))?;
   226|        let peers = room.peers.read().await;
   227|        let peer = peers.get(user_id)
   228|            .ok_or_else(|| format!("peer {} not found", user_id))?;
   229|
   230|        match IceCandidate::from_sdp(candidate) {
   231|            Ok(c) => peer.pc.add_ice_candidate(c)
   232|                .map_err(|e| format!("add_ice_candidate: {}", e)),
   233|            Err(e) => {
   234|                warn!(user=%user_id, "SFU invalid ICE candidate: {}", e);
   235|                Ok(())
   236|            }
   237|        }
   238|    }
   239|
   240|    /// Un participant quitte la room.
   241|    pub async fn remove_peer(
   242|        &self,
   243|        user_id: &str,
   244|        conversation_id: &str,
   245|    ) -> Result<Vec<String>, String> {
   246|        info!(user=%user_id, room=%conversation_id, "SFU remove_peer");
   247|        let rooms = self.rooms.read().await;
   248|        let room = rooms.get(conversation_id)
   249|            .ok_or_else(|| format!("room {} not found", conversation_id))?;
   250|
   251|        {
   252|            let mut peers = room.peers.write().await;
   253|            if let Some(p) = peers.remove(user_id) {
   254|                p.pc.close();
   255|            }
   256|        }
   257|
   258|        {
   259|            let mut tracks = room.tracks.write().await;
   260|            tracks.retain(|t| t.user_id != user_id);
   261|        }
   262|
   263|        {
   264|            let peers = room.peers.read().await;
   265|            for peer in peers.values() {
   266|                let mut added = peer.added_sources.write().await;
   267|                added.retain(|k| !k.starts_with(&format!("{}:", user_id)));
   268|            }
   269|        }
   270|
   271|        Ok({
   272|            let peers = room.peers.read().await;
   273|            peers.keys().cloned().collect()
   274|        })
   275|    }
   276|
   277|    /// Recuperer l offre de renegotiation pending d un peer.
   278|    pub async fn drain_pending_offer(&self, user_id: &str, conversation_id: &str) -> Option<SfuRenegotiateResponse> {
   279|        let rooms = self.rooms.read().await;
   280|        let room = rooms.get(conversation_id)?;
   281|        let peers = room.peers.read().await;
   282|        let peer = peers.get(user_id)?;
   283|
   284|        if peer.negotiation_pending.load(Ordering::SeqCst) {
   285|            peer.negotiation_pending.store(false, Ordering::SeqCst);
   286|            if let Some(desc) = peer.pc.local_description() {
   287|                info!(user=%user_id, "SFU pending offer drained");
   288|                return Some(SfuRenegotiateResponse {
   289|                    offer: desc.to_sdp_string(),
   290|                });
   291|            }
   292|        }
   293|        None
   294|    }
   295|
   296|    // ============================================================
   297|    // EVENT HANDLING
   298|    // ============================================================
   299|
   300|    fn setup_peer_events(pc: PeerConnection, room: Arc<Room>, peer: Arc<Peer>) {
   301|        let user_id = peer.user_id.clone();
   302|        let peer_id = peer.id;
   303|        let room_clone = room.clone();
   304|        let __peer_clone = peer.clone();
   305|
   306|        // Surveiller l etat ICE
   307|        let mut ice_rx = pc.subscribe_ice_connection_state();
   308|        let pc_for_ice = pc.clone();
   309|        let uid_ice = user_id.clone();
   310|        tokio::spawn(async move {
   311|            while let Ok(()) = ice_rx.changed().await {
   312|                let state = *ice_rx.borrow();
   313|                info!(user=%uid_ice, state=?state, "SFU ICE connection state");
   314|                match state {
   315|                    rustrtc::IceConnectionState::Disconnected
   316|                    | rustrtc::IceConnectionState::Failed
   317|                    | rustrtc::IceConnectionState::Closed => {
   318|                        info!(user=%uid_ice, "SFU closing PC on ICE disconnect");
   319|                        pc_for_ice.close();
   320|                        break;
   321|                    }
   322|                    _ => {}
   323|                }
   324|            }
   325|        });
   326|
   327|        // Boucle d evenements
   328|        tokio::spawn(async move {
   329|            while let Some(event) = pc.recv().await {
   330|                match event {
   331|                    PeerConnectionEvent::Track(transceiver) => {
   332|                        let receiver = match transceiver.receiver() {
   333|                            Some(r) => r,
   334|                            None => { warn!("SFU: no receiver on track event"); continue; }
   335|                        };
   336|                        let uid = user_id.clone();
   337|                        let track = receiver.track();
   338|                        let kind = track.kind();
   339|                        info!(user=%uid, kind=?kind, "SFU track received from peer");
   340|
   341|                        // Dedupliquer: une track par (user, kind)
   342|                        {
   343|                            let tracks = room_clone.tracks.read().await;
   344|                            if tracks.iter().any(|t| t.user_id == uid && t.kind == kind) {
   345|                                info!(user=%uid, kind=?kind, "SFU duplicate track, skipping");
   346|                                continue;
   347|                            }
   348|                        }
   349|
   350|                        // Creer un relay pour cette track
   351|                        let (clock_rate, payload_type, channels) = if kind == MediaKind::Video {
   352|                            (90_000u32, 96u8, 0u8)
   353|                        } else {
   354|                            (48_000u32, 111u8, 2u8)
   355|                        };
   356|
   357|                        let (source, local_track, _) = media::sample_track(kind, clock_rate as usize);
   358|                        let relay = MediaRelay::with_capacity(local_track.clone(), 500);
   359|
   360|                        let params = RtpCodecParameters {
   361|                            payload_type,
   362|                            clock_rate,
   363|                            channels,
   364|                        };
   365|
   366|                        let track_info = Arc::new(TrackInfo {
   367|                            relay,
   368|                            remote_track: track.clone(),
   369|                            user_id: uid.clone(),
   370|                            peer_id,
   371|                            kind,
   372|                            params: params.clone(),
   373|                        });
   374|
   375|                        // Ajouter aux tracks de la room
   376|                        {
   377|                            let mut tracks = room_clone.tracks.write().await;
   378|                            tracks.push(track_info.clone());
   379|                        }
   380|
   381|                        // Ajouter cette track aux AUTRES peers
   382|                        {
   383|                            let peers = room_clone.peers.read().await;
   384|                            for (other_id, other_peer) in peers.iter() {
   385|                                if *other_id == uid { continue; }
   386|                                let source_key = format!("{}:{}:{:?}", uid, peer_id, kind);
   387|                                {
   388|                                    let added = other_peer.added_sources.read().await;
   389|                                    if added.contains(&source_key) {
   390|                                        info!(to=%other_id, "SFU track already added, skip");
   391|                                        continue;
   392|                                    }
   393|                                }
   394|
   395|                                let relay_track = track_info.relay.subscribe();
   396|                                match other_peer.pc.add_track_with_stream_id(
   397|                                    relay_track,
   398|                                    track_info.user_id.clone(),
   399|                                    track_info.params.clone(),
   400|                                ) {
   401|                                    Ok(sender) => {
   402|                                        {
   403|                                            let mut added = other_peer.added_sources.write().await;
   404|                                            added.insert(source_key.clone());
   405|                                        }
   406|                                        info!(from=%uid, to=%other_id, kind=?kind, "SFU track relayed to peer");
   407|
   408|                                        // Forward PLI/RTCP de l autre peer vers la source
   409|                                        let remote_track = track.clone();
   410|                                        let mut rtcp_rx = sender.subscribe_rtcp();
   411|                                        let other_log = other_id.clone();
   412|                                        tokio::spawn(async move {
   413|                                            while let Ok(packet) = rtcp_rx.recv().await {
   414|                                                match packet {
   415|                                                    RtcpPacket::PictureLossIndication(_)
   416|                                                    | RtcpPacket::FullIntraRequest(_) => {
   417|                                                        info!(from=%other_log, "SFU forwarding PLI to source");
   418|                                                        let _ = remote_track.request_key_frame().await;
   419|                                                    }
   420|                                                    _ => {}
   421|                                                }
   422|                                            }
   423|                                        });
   424|
   425|                                        // Trigger renegotiation pour ce peer
   426|                                        Self::negotiate(other_peer.clone()).await;
   427|                                    }
   428|                                    Err(e) => {
   429|                                        warn!(to=%other_id, "SFU add_track failed: {}", e);
   430|                                    }
   431|                                }
   432|                            }
   433|                        }
   434|
   435|                        // Forward loop: track distante -> relay source
   436|                        let incoming = track.clone();
   437|                        let uid_fwd = uid.clone();
   438|                        tokio::spawn(async move {
   439|                            while let Ok(mut sample) = incoming.recv().await {
   440|                                match &mut sample {
   441|                                    media::MediaSample::Video(f) => {
   442|                                        f.header_extension = None;
   443|                                        f.payload_type = None;
   444|                                        if f.data.is_empty() { continue; }
   445|                                    }
   446|                                    media::MediaSample::Audio(f) => {
   447|                                        f.payload_type = None;
   448|                                        if f.data.is_empty() { continue; }
   449|                                    }
   450|                                }
   451|                                if source.send(sample).await.is_err() { break; }
   452|                            }
   453|                            info!(user=%uid_fwd, kind=?kind, "SFU forward loop ended");
   454|                        });
   455|
   456|                        // PLI periodique toutes les 3 secondes
   457|                        let pli_track = track.clone();
   458|                        let uid_pli = uid.clone();
   459|                        tokio::spawn(async move {
   460|                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
   461|                            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
   462|                            loop {
   463|                                interval.tick().await;
   464|                                if let Err(e) = pli_track.request_key_frame().await {
   465|                                    warn!(user=%uid_pli, "SFU PLI failed: {}", e);
   466|                                }
   467|                            }
   468|                        });
   469|                    }
   470|
   471|                    PeerConnectionEvent::DataChannel(_dc) => {
   472|                        // Chat/text est deja gere par le WS Nook
   473|                    }
   474|                }
   475|            }
   476|
   477|            // === Peer disconnect cleanup ===
   478|            info!(user=%user_id, "SFU peer event loop ended");
   479|            {
   480|                let mut tracks = room_clone.tracks.write().await;
   481|                tracks.retain(|t| !(t.user_id == user_id && t.peer_id == peer_id));
   482|            }
   483|            {
   484|                let mut peers = room_clone.peers.write().await;
   485|                if let Some(current) = peers.get(&user_id) {
   486|                    if current.id == peer_id {
   487|                        peers.remove(&user_id);
   488|                    }
   489|                }
   490|            }
   491|            {
   492|                for (_, other) in room_clone.peers.read().await.iter() {
   493|                    let mut added = other.added_sources.write().await;
   494|                    added.retain(|k| !k.starts_with(&format!("{}:{}:", user_id, peer_id)));
   495|                }
   496|            }
   497|        });
   498|    }
   499|
   500|    // ============================================================
   501|