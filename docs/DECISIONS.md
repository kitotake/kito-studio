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

## ADR-013 — Projet `.kito` : un seul fichier JSON versionné
Choix : fichier JSON UTF-8 unique (pas d'archive pour l'instant), `formatVersion` + `minReaderVersion`, migrations sur le JSON brut avant typage. Le champ `format` permettra de changer de conteneur plus tard. Spécification : `docs/KITO_FORMAT.md`.

## ADR-014 — Le document vit côté Rust
Choix : `project::session::Session` détient le document ; l'interface reçoit des instantanés (`ProjectSnapshot`) et envoie des commandes. Évite de transférer toute la timeline à chaque modification et prépare undo/redo. L'état d'enregistrement (`dirty`) est calculé côté Rust par révisions.

## ADR-015 — Temps en ticks entiers
Choix : 705 600 000 ticks par seconde, entiers bornés à 2^53 − 1. Divisible exactement par les cadences et fréquences audio courantes ; aucune dérive d'arrondi.

## ADR-016 — Conservation des données inconnues
Choix : `#[serde(flatten)] extra` à chaque niveau du modèle, effets/transitions/keyframes en JSON brut. Un fichier d'une version plus récente et compatible ne perd rien en passant par cette version.

## ADR-017 — Données déclaratives uniquement
Choix : aucun champ du `.kito` n'est exécutable. Effets, transitions et keyframes = objets `{ "type": "...", ... }` à identifiant strict ; profondeur, taille et références bornées. Le futur moteur de rendu n'interprétera que des types connus.

## ADR-018 — Enregistrement atomique et sauvegarde automatique séparée
Choix : temporaire voisin + fsync + `.bak` + renommage. Les copies automatiques vont dans `Autosave/` et n'écrasent jamais le fichier principal ; une copie plus récente que le fichier est proposée après un arrêt brutal ; elle est chargée comme travail non enregistré. Intervalle minimal 1 minute.

## ADR-019 — Médias : relatif d'abord, absolu ensuite, taille pour départager
Choix : chemins relatifs (séparateur `/`) recalculés à chaque enregistrement. Le candidat de même taille est préféré. Liaison automatique par dossier uniquement pour les correspondances non ambiguës (nom + taille).

## ADR-020 — Commandes de projet sur le pool de threads
Choix : `#[tauri::command(async)]` pour toutes les commandes qui touchent au disque (les commandes synchrones s'exécutent sur le thread principal de l'interface). L'écriture de la sauvegarde automatique se fait hors verrou.

## ADR-021 — Vitest et Testing Library
Choix : tests React avec Vitest (même configuration que Vite) et Testing Library en jsdom ; l'API native est simulée au niveau du module `@/lib/tauri`. Voir `docs/TESTING.md`.

