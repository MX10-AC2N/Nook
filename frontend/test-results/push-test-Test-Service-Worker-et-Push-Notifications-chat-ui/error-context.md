# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: push-test.spec.ts >> Test Service Worker et Push Notifications
- Location: tests/push-test.spec.ts:5:1

# Error details

```
Test timeout of 60000ms exceeded.
```

```
Error: page.evaluate: Test timeout of 60000ms exceeded.
```

# Page snapshot

```yaml
- generic [ref=e2]:
  - banner [ref=e3]:
    - button "Ouvrir le menu de navigation" [ref=e4] [cursor=pointer]: ☰
    - heading "Logo animé NOOK - Style Encre" [level=1] [ref=e5]:
      - img "Logo animé NOOK - Style Encre" [ref=e7]:
        - generic [ref=e12]: "N"
        - generic [ref=e13]: O
        - generic [ref=e14]: O
        - generic [ref=e15]: K
    - generic [ref=e16]: Hermes
    - button "Déconnexion" [ref=e17] [cursor=pointer]:
      - img [ref=e19]
  - main [ref=e21]:
    - generic [ref=e22]:
      - heading "Paramètres" [level=1] [ref=e24]:
        - img [ref=e26]
        - text: Paramètres
      - tablist [ref=e28]:
        - tab "Profil" [ref=e29] [cursor=pointer]
        - tab "Sécurité" [ref=e30] [cursor=pointer]:
          - img [ref=e32]
          - text: Sécurité
        - tab "Apparence" [ref=e34] [cursor=pointer]
      - generic [ref=e35]:
        - generic [ref=e36]:
          - generic [ref=e37]: Avatar
          - paragraph [ref=e38]: Choisissez un style puis sélectionnez votre avatar parmi les propositions.
          - img "Hermes" [ref=e40]
          - generic [ref=e41]: Sélectionner un style
          - generic [ref=e42]:
            - button "Aventurier Aventurier" [ref=e43] [cursor=pointer]:
              - img "Aventurier" [ref=e44]
              - generic [ref=e45]: Aventurier
            - button "Cartoon Cartoon" [ref=e46] [cursor=pointer]:
              - img "Cartoon" [ref=e47]
              - generic [ref=e48]: Cartoon
            - button "Illustré Illustré" [ref=e49] [cursor=pointer]:
              - img "Illustré" [ref=e50]
              - generic [ref=e51]: Illustré
            - button "Minimaliste Minimaliste" [ref=e52] [cursor=pointer]:
              - img "Minimaliste" [ref=e53]
              - generic [ref=e54]: Minimaliste
            - button "Emoji Emoji" [ref=e55] [cursor=pointer]:
              - img "Emoji" [ref=e56]
              - generic [ref=e57]: Emoji
            - button "Sourire Sourire" [ref=e58] [cursor=pointer]:
              - img "Sourire" [ref=e59]
              - generic [ref=e60]: Sourire
            - button "Portrait Portrait" [ref=e61] [cursor=pointer]:
              - img "Portrait" [ref=e62]
              - generic [ref=e63]: Portrait
            - button "Personas Personas" [ref=e64] [cursor=pointer]:
              - img "Personas" [ref=e65]
              - generic [ref=e66]: Personas
            - button "Robot Robot" [ref=e67] [cursor=pointer]:
              - img "Robot" [ref=e68]
              - generic [ref=e69]: Robot
            - button "Initiales Initiales" [ref=e70] [cursor=pointer]:
              - img "Initiales" [ref=e71]
              - generic [ref=e72]: Initiales
          - generic [ref=e73]: Choisissez votre avatar
          - generic [ref=e74]:
            - button "Theo" [ref=e75] [cursor=pointer]:
              - img "Theo" [ref=e76]
            - button "Ziggy" [ref=e77] [cursor=pointer]:
              - img "Ziggy" [ref=e78]
            - button "Nina" [ref=e79] [cursor=pointer]:
              - img "Nina" [ref=e80]
            - button "Ivy" [ref=e81] [cursor=pointer]:
              - img "Ivy" [ref=e82]
            - button "Kiki" [ref=e83] [cursor=pointer]:
              - img "Kiki" [ref=e84]
            - button "Juno" [ref=e85] [cursor=pointer]:
              - img "Juno" [ref=e86]
            - button "Ash" [ref=e87] [cursor=pointer]:
              - img "Ash" [ref=e88]
            - button "Cosmo" [ref=e89] [cursor=pointer]:
              - img "Cosmo" [ref=e90]
            - button "Jazz" [ref=e91] [cursor=pointer]:
              - img "Jazz" [ref=e92]
            - button "Spark" [ref=e93] [cursor=pointer]:
              - img "Spark" [ref=e94]
            - button "Sol" [ref=e95] [cursor=pointer]:
              - img "Sol" [ref=e96]
            - button "Nova" [ref=e97] [cursor=pointer]:
              - img "Nova" [ref=e98]
            - button "Zoe" [ref=e99] [cursor=pointer]:
              - img "Zoe" [ref=e100]
            - button "Nori" [ref=e101] [cursor=pointer]:
              - img "Nori" [ref=e102]
            - button "Mika" [ref=e103] [cursor=pointer]:
              - img "Mika" [ref=e104]
            - button "Bree" [ref=e105] [cursor=pointer]:
              - img "Bree" [ref=e106]
            - button "Oscar" [ref=e107] [cursor=pointer]:
              - img "Oscar" [ref=e108]
            - button "Felix" [ref=e109] [cursor=pointer]:
              - img "Felix" [ref=e110]
            - button "Pixel" [ref=e111] [cursor=pointer]:
              - img "Pixel" [ref=e112]
            - button "Vega" [ref=e113] [cursor=pointer]:
              - img "Vega" [ref=e114]
          - button "🔄 Autres propositions" [ref=e115] [cursor=pointer]
        - heading "Informations du profil" [level=2] [ref=e116]
        - generic [ref=e117]:
          - generic [ref=e118]:
            - generic [ref=e119]: Prénom / Nom affiché
            - textbox "Prénom / Nom affiché" [ref=e120]:
              - /placeholder: Votre prénom
              - text: Hermes
            - paragraph [ref=e121]: Ce nom sera visible par les autres membres
          - generic [ref=e122]:
            - generic [ref=e123]: Identifiant de connexion
            - textbox "Identifiant de connexion" [disabled] [ref=e124]: hermes-bot
            - paragraph [ref=e125]:
              - text: ⚠️ L'identifiant de connexion ne peut pas être modifié. Vous devez toujours utiliser
              - strong [ref=e126]: hermes-bot
              - text: pour vous connecter, même si vous avez changé votre nom affiché.
          - button "Enregistrer" [ref=e127] [cursor=pointer]
  - contentinfo [ref=e128]:
    - paragraph [ref=e129]: © 2026 Nook • Messagerie privée pour la famille
```

# Test source

```ts
  1   | import { test } from '@playwright/test';
  2   | 
  3   | test.use({ ignoreHTTPSErrors: true });
  4   | 
  5   | test('Test Service Worker et Push Notifications', async ({ page }) => {
  6   |   const logs: string[] = [];
  7   |   const errors: string[] = [];
  8   |   
  9   |   page.on('console', msg => {
  10  |     logs.push(msg.text());
  11  |     if (msg.type() === 'error') errors.push(msg.text());
  12  |   });
  13  |   
  14  |   // Login
  15  |   await page.goto('https://192.168.1.192:6443/login');
  16  |   await page.waitForTimeout(2000);
  17  |   await page.fill('input[type="text"], input[name="username"]', 'hermes-bot');
  18  |   await page.fill('input[type="password"]', 'Hermes2026!');
  19  |   await page.click('button[type="submit"]');
  20  |   await page.waitForTimeout(3000);
  21  |   
  22  |   console.log('URL après login:', page.url());
  23  |   
  24  |   // Aller aux paramètres
  25  |   await page.goto('https://192.168.1.192:6443/settings');
  26  |   await page.waitForTimeout(3000);
  27  |   
  28  |   // Vérifier Service Worker
  29  |   const swInfo = await page.evaluate(async () => {
  30  |     if (!('serviceWorker' in navigator)) return { supported: false };
  31  |     try {
  32  |       const reg = await Promise.race([
  33  |         navigator.serviceWorker.ready,
  34  |         new Promise<null>((_, reject) => setTimeout(() => reject('timeout'), 10000))
  35  |       ]);
  36  |       return {
  37  |         supported: true,
  38  |         registered: !!reg,
  39  |         active: !!reg?.active,
  40  |         scope: reg?.scope
  41  |       };
  42  |     } catch (e) {
  43  |       return { supported: true, error: String(e) };
  44  |     }
  45  |   });
  46  |   
  47  |   console.log('\n=== SERVICE WORKER ===');
  48  |   console.log('Supported:', swInfo.supported ? '✅' : '❌');
  49  |   console.log('Registered:', swInfo.registered ? '✅' : '❌');
  50  |   console.log('Active:', swInfo.active ? '✅' : '❌');
  51  |   if (swInfo.scope) console.log('Scope:', swInfo.scope);
  52  |   if (swInfo.error) console.log('Error:', swInfo.error);
  53  |   
  54  |   // Vérifier Push Manager
> 55  |   const pushInfo = await page.evaluate(async () => {
      |                               ^ Error: page.evaluate: Test timeout of 60000ms exceeded.
  56  |     if (!('PushManager' in window)) return { supported: false };
  57  |     try {
  58  |       const reg = await navigator.serviceWorker.ready;
  59  |       const sub = await reg.pushManager.getSubscription();
  60  |       return {
  61  |         supported: true,
  62  |         subscribed: !!sub
  63  |       };
  64  |     } catch (e) {
  65  |       return { supported: true, error: String(e) };
  66  |     }
  67  |   });
  68  |   
  69  |   console.log('\n=== PUSH MANAGER ===');
  70  |   console.log('Supported:', pushInfo.supported ? '✅' : '❌');
  71  |   console.log('Subscribed:', pushInfo.subscribed ? '✅' : '❌');
  72  |   if (pushInfo.error) console.log('Error:', pushInfo.error);
  73  |   
  74  |   // Vérifier VAPID
  75  |   const vapidInfo = await page.evaluate(async () => {
  76  |     try {
  77  |       const r = await fetch('/api/vapid-public-key');
  78  |       const key = await r.text();
  79  |       return { ok: r.ok, key, length: key.length };
  80  |     } catch (e) {
  81  |       return { ok: false, error: String(e) };
  82  |     }
  83  |   });
  84  |   
  85  |   console.log('\n=== VAPID KEY ===');
  86  |   console.log('Endpoint:', vapidInfo.ok ? '✅ OK' : '❌ FAILED');
  87  |   if (vapidInfo.key) {
  88  |     console.log('Key (first 40):', vapidInfo.key.slice(0, 40) + '...');
  89  |     console.log('Length:', vapidInfo.length, 'chars');
  90  |     const isBase64url = /^[A-Za-z0-9_-]+$/.test(vapidInfo.key);
  91  |     console.log('Format base64url:', isBase64url ? '✅' : '❌');
  92  |   }
  93  |   
  94  |   // Logs SW
  95  |   console.log('\n=== SW LOGS ===');
  96  |   const swLogs = logs.filter(l => l.includes('[SW]'));
  97  |   for (const l of swLogs.slice(0, 10)) console.log('  ' + l);
  98  |   
  99  |   // Erreurs
  100 |   if (errors.length > 0) {
  101 |     console.log('\n=== ERRORS ===');
  102 |     for (const e of errors.slice(0, 10)) console.log('  ' + e);
  103 |   }
  104 | });
  105 | 
```