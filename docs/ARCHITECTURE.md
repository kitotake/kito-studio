# Architecture

## Vue d'ensemble

```
React (src/)  ──nativeApi──▶  commandes Tauri (src-tauri/src/commands/)  ──▶  modules métier Rust
   interface                       frontière fine (AppResult)                  (indépendants de Tauri)
```

L'interface ne manipule jamais FFmpeg, le disque ni les structures du moteur : elle appelle une API native typée. Le backend orchestre ; les modules métier sont testables sans Tauri.

## Frontend (`src/`)

| Dossier | Rôle | Peut importer |
|---------|------|---------------|
| `types/` | Types partagés (projet, système, erreurs, navigation) | rien |
| `lib/` | Utilitaires transverses : `errors.ts` (`KitoError`), `tauri/` (client + `nativeApi`), `appInfo.ts` | `types` |
| `components/ui/` | Briques génériques sans logique métier (`Modal`, `Panel`) | `types` |
| `components/layout/` | Structure de l'application (`Sidebar`, `Topbar`) | `ui`, `features/system`, `lib`, `types` |
| `features/<nom>/` | Une fonctionnalité = ses écrans, composants, hooks et logique | `components`, `lib`, `types`, autres `features` via leur API publique |
| `app/` | Assemblage final : `App`, `ErrorBoundary`, `useWorkspace` | tout |
| `styles/` | SCSS global organisé (voir ci-dessous) | — |

Règle : les dépendances vont toujours vers le bas (`app` → `features` → `components` → `lib` → `types`). Jamais l'inverse. Alias d'import : `@/` = `src/`.

Features actuelles : `home`, `project` (dialogue et profil de création), `editor` (écran et 4 panneaux), `system` (état du backend partagé via contexte).

### Styles (`src/styles/`)

`abstracts/` (tokens) → `base/` (reset, typographie) → `layout/` → `components/` → `features/<nom>/` → `_responsive.scss`.
Le fichier `main.scss` définit l'ordre des `@forward`. **Cet ordre est significatif** (cascade CSS) ; `_responsive.scss` reste toujours en dernier.

## Backend (`src-tauri/src/`)

| Fichier / dossier | Rôle |
|-------------------|------|
| `lib.rs` | Déclare les modules et enregistre les commandes |
| `error.rs` | `AppError` / `AppResult`, sérialisé en `{ code, message }` |
| `commands/` | Frontière Tauri : valide l'entrée, délègue, retourne `AppResult`. Aucune logique métier |
| `system.rs`, `media/` | Modules métier sans dépendance à Tauri, avec tests unitaires |

### Contrat d'erreur

Toute commande retourne `AppResult<T>`. En cas d'échec, l'interface reçoit `{ code, message }` et `invokeCommand` le convertit en `KitoError`. Codes Rust : `IO`, `INVALID_INPUT`, `NOT_FOUND`, `INTERNAL` ; codes côté interface : `UNAVAILABLE` (hors application native), `UNKNOWN`. Les deux listes (`error.rs` et `src/types/errors.ts`) doivent rester synchronisées. Les erreurs de rendu React sont rattrapées par `ErrorBoundary`.

### Ajouter une fonctionnalité

1. Rust : créer le module métier (ex. `project/`) avec ses tests, puis `commands/project.rs` (fonctions fines retournant `AppResult`), puis l'enregistrer dans `generate_handler!`.
2. Frontend : ajouter les types dans `types/`, une entrée dans `nativeApi`, puis un dossier `features/<nom>/`.
3. Styles : un partiel dans `styles/features/<nom>/` et un `@forward` dans `main.scss` avant `responsive`.

## Correspondance avec les modules prévus

| Module visé | Rust | Frontend | État |
|-------------|------|----------|------|
| Project | — | `features/project` (création en mémoire) | à faire : `.kito` |
| Media | `media/tools.rs` (détection FFmpeg) | `features/editor/panels/MediaPanel` (vide) | import/probe à faire |
| Timeline | — | `features/editor/panels/TimelinePanel` (statique) | à faire |
| Playback | — | `features/editor/panels/PreviewPanel` (statique) | à faire |
| Render, Export, Audio, Effects, Keyframes, Cache, Sync, AI | — | — | pas commencés, aucun code factice |

Chaque module futur reçoit un module Rust et un dossier `features/` au moment où il est réellement implémenté.

## Stratégie FFmpeg

Étape initiale : détecter `ffmpeg` et `ffprobe` sans les exécuter.

Étape suivante : introduire un adaptateur média Rust qui lance les binaires avec des arguments séparés (jamais via une chaîne de commande shell), contrôle les chemins, capture les erreurs et permet l'annulation des tâches.

Avant la distribution, décider entre binaires FFmpeg externes, bibliothèques dynamiques ou compilation dédiée. Documenter la configuration de compilation et la conformité de licence.

## Projet `.kito`

Le format sera versionné. Le modèle logique sera indépendant du conteneur physique. Première cible : un manifeste JSON validé, puis un conteneur d'archive avec ressources facultatives.

## Performance

- Les traitements lourds ne doivent jamais bloquer l'interface.
- Utiliser des tâches asynchrones et des canaux d'événements.
- Mettre en cache les miniatures, proxys et formes d'onde.
- Prévoir une politique de mémoire et de cache configurable.
