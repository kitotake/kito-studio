# Décisions d'architecture

## ADR-001 — Tauri 2
Choix : Tauri 2 pour partager le frontend et conserver un backend natif.

## ADR-002 — Rust
Choix : Rust pour les opérations natives, la gestion des projets et l'orchestration du moteur.

## ADR-003 — FFmpeg
Choix : FFmpeg pour l'analyse, le décodage, le traitement et l'encodage média. Les modalités de distribution restent à définir après revue des licences.

## ADR-004 — Windows en premier
Choix : valider le flux de travail desktop avant d'activer Android.

## ADR-005 — Local-first
Choix : les projets doivent être utilisables sans compte ni cloud. La synchronisation sera optionnelle.

## ADR-006 — Projets TypeScript séparés
Choix : `tsconfig.app.json` (code de l'application) et `tsconfig.node.json` (configuration Vite), référencés par `tsconfig.json` et vérifiés ensemble par `tsc -b`.

## ADR-007 — Scripts de build pnpm explicitement autorisés
Choix : seul `esbuild` est autorisé à exécuter son script d'installation (`pnpm-workspace.yaml`). Toute nouvelle dépendance avec script natif doit être approuvée explicitement.

## ADR-008 — Deux politiques CSP
Choix : `csp` stricte en production, `devCsp` assouplie (inline + WebSocket) uniquement pour le serveur de développement Vite.

## ADR-009 — Organisation par fonctionnalités et alias `@/`
Choix : `features/<nom>/` pour chaque fonctionnalité, `components/` pour le générique, dépendances uniquement vers le bas. Alias `@/` déclaré dans `tsconfig.app.json` et `vite.config.ts` (à garder synchronisés). Les styles restent globaux (mêmes noms de classes) mais sont rangés par couche et par fonctionnalité.

## ADR-010 — Contrat d'erreur unique Rust ↔ React
Choix : toute commande retourne `AppResult<T>` ; l'erreur est sérialisée en `{ code, message }` puis convertie en `KitoError` par `invokeCommand`. Aucune dépendance ajoutée (pas de `thiserror`). Les composants n'appellent jamais `invoke` directement.

## ADR-011 — La vue est distincte du projet ouvert
Choix : `useWorkspace` sépare `view` (accueil / éditeur) de `project`. Avant, cliquer sur « Accueil » détruisait le projet en mémoire ; maintenant il est conservé.

## ADR-012 — Pas de bibliothèque d'état global pour l'instant
Choix : `useState` + un contexte (`SystemStatus`). À réévaluer quand la timeline et le projet courant seront partagés par de nombreux composants (candidat : store léger avec historique undo/redo).

