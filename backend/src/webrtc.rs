// backend/src/webrtc.rs
// Signalisation P2P + Chiffrement fichiers (XChaCha20-Poly1305)
// Session 9  — fix sécurité : authentification du WebSocket
//   → le cookie auth_token est vérifié dès la connexion WS
//   → connexion refusée si token invalide ou manquant
// Session 36 — SEC-05 : limite 64 KB sur les messages WS de signaling

use axum::{
    extract::{ws::WebSocket, Json as AxumJson, State as AxumState},
    http::{header::COOKIE, StatusCode},
    response::IntoResponse,
};
use base64ct::{Base64Unpadded, Encoding};
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::KeyInit;
use chacha20poly1305::XChaCha20Poly1305;
use futures_util::{SinkExt, StreamExt};
use rand::RngCore;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use crate::SharedState;
use tokio::time::{interval, sleep};
use uuid::Uuid;

// ════════════════════════════════════════════════════════════════
// CRYPTO — Compatible libsodium (XChaCha20-Poly1305, nonces 24 bytes)
// ════════════════════════════════════════════════════════════════

const CRYPTO_SECRETBOX_NONCEBYTES: usize = 24;
const CRYPTO_SECRETBOX_KEYBYTES: usize = 32;
const CRYPTO_SECRETBOX_MACBYTES: usize = 16;

const FILE_EXPIRATION_HOURS: u64 = 48;
const CLEANUP_INTERVAL_HOURS: u64 = 1;

fn crypto_secretbox_keygen() -> Vec<u8> {
    let mut key = vec![0u8; CRYPTO_SECRETBOX_KEYBYTES];
    rand::rng().fill_bytes(&mut key);
    key
}

fn crypto_secretbox_nonce() -> Vec<u8> {
    let mut nonce = vec![0u8; CRYPTO_SECRETBOX_NONCEBYTES];
    rand::rng().fill_bytes(&mut nonce);
    nonce
}

fn crypto_secretbox_easy(message: &[u8], key: &[u8], nonce: &[u8]) -> Vec<u8> {
    let mut result = Vec::