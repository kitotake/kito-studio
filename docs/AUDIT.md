# Audit initial et corrections — Étape 1 (Fondations)

Date : 15/11/2026. Base : `kito-studio-initial.zip`.

## Problèmes bloquants trouvés (reproduits avant correction)

| # | Problème | Symptôme | Correction |
|---|----------|----------|------------|
| 1 | `global.scss` importait `./components`, fichier inexistant | `pnpm build` / `pnpm dev` : « Can't find stylesheet to import » | Import supprimé (tous les styles sont dans `_layout.scss`) |
| 2 | Dossier `src-tauri/icons/` absent | `cargo check` : « failed to open icon icons/icon.png » ; `icon.ico` requis par Windows | Icônes générées avec `tauri icon` depuis `branding/app-icon.png` + `bundle.icon` déclaré |
| 3 | `pnpm install` échoue avec pnpm récent | `ERR_PNPM_IGNORED_BUILDS` (esbuild) | `onlyBuiltDependencies` (pnpm 10) et `allowBuilds` (pnpm 11+) dans `pnpm-workspace.yaml` |

## Problèmes non bloquants corrigés

- `.gitignore` ignorait `src-tauri/capabilities/*.json` : `default.json` (obligatoire pour les permissions Tauri 2) n'aurait pas été versionné. Règle retirée.
- `tsconfig.node.json` était invalide (TS5096) et jamais vérifié. Config scindée en `tsconfig.app.json` + `tsconfig.node.json`, référencées par `tsconfig.json` ; `tsc -b` vérifie maintenant aussi `vite.config.ts`.
- CSP : ajout de `connect-src ipc: http://ipc.localhost` (requis par l'IPC Tauri 2) et d'une `devCsp` distincte (inline scripts + WebSocket de Vite pour le HMR).

## Résultats de test

| Vérification | Résultat | Environnement |
|--------------|----------|---------------|
| `pnpm install` (pnpm 10.34 et 12.21) | OK | Linux, Node 22 |
| `pnpm build` (`tsc -b` + Vite) | OK | Linux |
| `cargo check` / `cargo build` (Rust 1.97, Tauri 2.12) | OK | Linux |
| Lancement de l'app native + IPC (`get_runtime_info`, `check_media_tools`) | OK, vérifié par capture d'écran | Linux, Xvfb |
| `pnpm tauri dev` / `pnpm tauri build` (NSIS) sous Windows | **NON TESTÉ** | — |

Le build Windows n'a pas pu être exécuté dans l'environnement d'audit. La case « Premier build Windows validé » de la roadmap reste donc décochée.

## Points d'attention restants (non traités volontairement)

- `src/App.tsx` (~10 Ko) contient tout : accueil, éditeur, modale, état. À découper à l'étape 1 suite (composants, hooks, store).
- `src/styles/_layout.scss` (~11 Ko) est dense (règles sur une seule ligne) et mélange tous les composants. À scinder par composant.
- Aucune gestion d'erreur structurée côté Rust (`Result` + type d'erreur sérialisable) : à mettre en place avant d'ajouter des commandes qui touchent au disque.
- `check_media_tools` ne cherche que dans le `PATH`. Pour la distribution, décider entre sidecar Tauri (binaires FFmpeg embarqués) et FFmpeg externe.
- Le logo d'application est un placeholder généré à partir du « K » de l'interface.
- Les modules Effects, Keyframes, Cache, Sync et AI ne figurent pas encore dans `docs/ARCHITECTURE.md`.
- Dans l'interface, « bibliothèque de médias » et « timeline » sont des états vides / emplacements statiques, pas de données de démonstration.

## Correctif ultérieur
L'archive `kito-studio-etape1.zip` contenait par erreur des fichiers émis par un test `tsc` (`vite.config.js`, `vite.config.d.ts`, `*.tsbuildinfo`). Vite charge `vite.config.js` avant `vite.config.ts` : toute modification du `.ts` aurait été ignorée. Fichiers supprimés et ajoutés au `.gitignore`.

