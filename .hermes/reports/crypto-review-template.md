# 🔐 Revue Crypto — {component}

> **Profil**: security-auditor
> **Date**: {date}
> **Fichier**: `{file_path}`

---

## Vérifications Effectuées

### Crypto Hygiene
| Vérification | Résultat | Détail |
|-------------|----------|--------|
| unwrap/expect dans crypto | {unwrap_status} | {unwrap_detail} |
| Constant-time comparisons | {ct_status} | {ct_detail} |
| Nonce XChaCha20 (192-bit) | {nonce_status} | {nonce_detail} |
| Keys zeroized on drop | {zeroize_status} | {zeroize_detail} |
| rand_core 0.6 | {rand_status} | {rand_detail} |
| Argon2id params | {argon2_status} | {argon2_detail} |

### Algorithm Verification
| Algorithm | Usage | Status | File |
|-----------|-------|--------|------|
| X25519 | ECDH key agreement | {x25519_status} | `{file}` |
| Ed25519 | Signatures | {ed25519_status} | `{file}` |
| XChaCha20-Poly1305 | Symmetric | {xchacha_status} | `{file}` |
| HKDF-SHA256 | KDF | {hkdf_status} | `{file}` |

---

## Menaces Identifiées
{threats}

---

## Conclusion
{conclusion}
