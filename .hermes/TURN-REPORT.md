# 🧪 Turn-Server Test Report — amd64

> Généré par `test-turn.yml` · **2026-06-09 05:25 UTC**
> Architecture: **amd64** | Run: [27185597706](https://github.com/MX10-AC2N/Nook/actions/runs/27185597706)

---

## 📊 Résumé

| Test | Résultat |
|------|----------|
| Build Docker | ✅ |
| Container start | ✅ |
| Port 3478 UDP | PASS |
| Port 3478 TCP | PASS |
| STUN Binding | ✅ PASS |
| TURN Allocate (no auth) | ✅ PASS |
| TURN Allocate (with auth) | ❌ FAIL |

---

## 🐳 Container Info

```
Image: nook-turn:test
Status: running
Started: 2026-06-09T05:25:07.0809019Z
Ports: {"3478/tcp":[{"HostIp":"0.0.0.0","HostPort":"3478"},{"HostIp":"::","HostPort":"3478"}],"3478/udp":[{"HostIp":"0.0.0.0","HostPort":"3478"},{"HostIp":"::","HostPort":"3478"}]}
```

## 📋 Container Logs

```

```

## 🔧 Config Used

```toml
[server]
name = "nook.turn.test"
secret = "test_secret_123"
max-threads = 4

[[server.interfaces]]
transport = "udp"
listen = "0.0.0.0:3478"
external = "0.0.0.0:3478"

[[server.interfaces]]
transport = "tcp"
listen = "0.0.0.0:3478"
external = "0.0.0.0:3478"
```

## 🌐 Network

```
udp   UNCONN 0      0              0.0.0.0:3478      0.0.0.0:*          
udp   UNCONN 0      0                 [::]:3478         [::]:*          
tcp   LISTEN 0      4096           0.0.0.0:3478      0.0.0.0:*          
tcp   LISTEN 0      4096              [::]:3478         [::]:*          
```

## 🖥️ System Info

```
Linux runnervm3jyl0 6.17.0-1015-azure #15~24.04.1-Ubuntu SMP Wed May  6 22:37:49 UTC 2026 x86_64 x86_64 x86_64 GNU/Linux
Docker: Docker version 28.0.4, build b8034c0
```

---

*Rapport généré par `.github/workflows/test-turn.yml`*
