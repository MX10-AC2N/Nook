# npm ci / package-lock.json mismatch — Fix rapide

## Symptôme
```
npm ci
npm error code EINTEGRITY
npm error sha512-... does not match
```

Ou dans CI : `npm ci` échoue alors que `npm install` passe en local.

## Cause
- `package-lock.json` dans le repo ne correspond pas aux versions résolues par `npm install` local
- Souvent causé par `npm install` sans commit du lockfile, ou différences de registre/npm version

## Fix validé (Session 53)
```bash
cd /opt/data/Nook/frontend
rm package-lock.json
rm -rf node_modules
npm install
git add package-lock.json
git commit -m "chore(frontend): régénérer package-lock.json"
git push
```

## Alternative si `npm ci` requis en CI
```bash
# Réinstaller proprement
npm ci --prefer-offline --no-audit --no-fund
# Ou forcer la régénération
npm install --package-lock-only
```

## Note importante pour Nook
Le `package-lock.json` **doit être committé** dans le repo (gitignore ne l'exclut pas).
Toute modification de `package.json` → régénérer + commit le lockfile.