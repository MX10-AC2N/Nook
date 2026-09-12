# 🚨 Plan d'Incident Response — Nook

> **Profil**: security-auditor
> **Dernière mise à jour**: {date}
> **Version**: {version}

---

## Équipe d'Incident

| Rôle | Responsable | Contact |
|------|-------------|---------|
| Lead Security | security-auditor | — |
| Dev Lead | coder | — |
| DevOps | deployer | — |

---

## Protocoles par Type d'Incident

### 🔐 Compromission E2EE
1. Isoler immédiatement les clés concernées
2. Forcer la rotation des clés (DT-05)
3. Révoquer les tokens des utilisateurs affectés
4. Auditer les logs pour détecter l'exfiltration
5. Informer les utilisateurs si les messages étaient en clair

### 🛡️ Faille Rate Limiting (DT-04)
1. Activer les limites d'urgence via Nginx
2. Augmenter les seuils temporairement si nécessaire
3. Vérifier le StateStore Redis
4. Auditer les logs pour détecter les abus

### 📦 Vulnérabilité Dépendance
1. Identifier le package vulnérable (`cargo audit`)
2. Évaluer l'impact
3. Patcher ou isoler
4. Redéployer si nécessaire

### 🔓 Fuite de Données
1. Couper l'accès externe
2. Identifier la source de la fuite
3. Évaluer l'impact (quels données exposées)
4. Notifier les autorités si nécessaire (RGPD)
5. Réparer et redéployer

---

## Checklist Post-Incident
- [ ] Root cause identified
- [ ] Fix deployed
- [ ] Tests pass
- [ ] Monitoring restored
- [ ] Post-mortem written
- [ ] PLUR engram written (type=procedural)
- [ ] Threat model updated if needed

---

## Contacts d'Urgence
{contacts}
