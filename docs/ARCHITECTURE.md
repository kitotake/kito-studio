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

Features actuelles :

- `project` : tout ce qui concerne le projet ouvert — `ProjectProvider` (instantané du backend, occupation, erreur courante, copie de récupération, état de sauvegarde automatique), dialogues (création, paramètres, préférences, médias manquants, récupération, modifications non enregistrées), liste des récents, indicateur de sauvegarde, barre d'outils, hooks (`useUnsavedGuard`, `useCloseGuard`, `useProjectShortcuts`, `useRecentProjects`), messages d'erreur compréhensibles (`projectErrors.ts`).
- `home` (accueil), `editor` (écran et 4 panneaux), `system` (état du backend partagé via contexte).
- `app/Workspace.tsx` orchestre les flux (créer, ouvrir, enregistrer, fermer, récupérer) ; la navigation seule est dans `useWorkspace`.

### Styles (`src/styles/`)

`abstracts/` (tokens) → `base/` (reset, typographie) → `layout/` → `components/` → `features/<nom>/` → `_responsive.scss`.
Le fichier `main.scss` définit l'ordre des `@forward`. **Cet ordre est significatif** (cascade CSS) ; `_responsive.scss` reste toujours en dernier.

## Backend (`src-tauri/src/`)

| Fichier / dossier | Rôle |
|-------------------|------|
| `lib.rs` | Déclare les modules, le plugin de dialogue, l'état géré, la boucle d'autosave, les commandes |
| `error.rs` | `AppError` / `AppResult`, sérialisé en `{ code, message }` |
| `state.rs` | État géré par Tauri : projet ouvert, préférences, dossier de données |
| `commands/` | Frontière Tauri (`system`, `project`, `preferences`, `autosave`) : valide l'entrée, délègue, retourne `AppResult`. Aucune logique métier |
| `project/` | Module Project (sans dépendance à Tauri) : `model` (format), `validate`, `migrate`, `io` (lecture validée, écriture atomique), `paths`, `media` (résolution portable), `names`, `autosave`, `recent`, `session` (document ouvert), `snapshot` (vues pour l'interface) |
| `preferences.rs` | Préférences versionnées (`preferences.json`) |
| `system.rs`, `media/` | Informations système, détection FFmpeg |
| `timeutil.rs` | Horodatages UTC sans dépendance |

Les commandes qui touchent au disque sont déclarées `#[tauri::command(async)]` : elles s'exécutent sur le pool de threads et jamais sur le thread de l'interface. La sauvegarde automatique tourne dans un thread dédié (`commands/autosave.rs`) : décision et sérialisation sous le verrou, écriture disque hors verrou.

### Contrat d'erreur

Toute commande retourne `AppResult<T>`. En cas d'échec, l'interface reçoit `{ code, message }` et `invokeCommand` le convertit en `KitoError`. Codes Rust : `IO`, `INVALID_INPUT`, `NOT_FOUND`, `ALREADY_EXISTS`, `PERMISSION_DENIED`, `DISK_FULL`, `INVALID_FORMAT`, `INCOMPATIBLE_VERSION`, `INTERNAL` ; `AppError::from_io` choisit le code le plus précis selon l'erreur système. Codes côté interface : `UNAVAILABLE` (hors application native), `UNKNOWN`. Les deux listes (`error.rs` et `src/types/errors.ts`) doivent rester synchronisées. `projectErrors.ts` transforme chaque code en titre + message + conseil. Les erreurs de rendu React sont rattrapées par `ErrorBoundary`.

### Ajouter une fonctionnalité

1. Rust : créer le module métier (ex. `project/`) avec ses tests, puis `commands/project.rs` (fonctions fines retournant `AppResult`), puis l'enregistrer dans `generate_handler!`.
2. Frontend : ajouter les types dans `types/`, une entrée dans `nativeApi`, puis un dossier `features/<nom>/`.
3. Styles : un partiel dans `styles/features/<nom>/` et un `@forward` dans `main.scss` avant `responsive`.

## Correspondance avec les modules prévus

| Module visé | Rust | Frontend | État |
|-------------|------|----------|------|
| Project | `project/` (format, session, sauvegardes, récents) | `features/project` | **fait** : créer, ouvrir, enregistrer, autosave, récupération, médias manquants |
| Media | `media/tools.rs` (détection FFmpeg) ; `project/media.rs` (résolution des chemins) | `features/editor/panels/MediaPanel` (liste et état des médias du projet) | import/probe/miniatures à faire |
| Timeline | — | `features/editor/panels/TimelinePanel` (statique) | à faire |
| Playback | — | `features/editor/panels/PreviewPanel` (statique) | à faire |
| Render, Export, Audio, Effects, Keyframes, Cache, Sync, AI | — | — | pas commencés, aucun code factice |

Chaque module futur reçoit un module Rust et un dossier `features/` au moment où il est réellement implémenté.

## Stratégie FFmpeg

Étape initiale : détecter `ffmpeg` et `ffprobe` sans les exécuter.

Étape suivante : introduire un adaptateur média Rust qui lance les binaires avec des arguments séparés (jamais via une chaîne de commande shell), contrôle les chemins, capture les erreurs et permet l'annulation des tâches.

Avant la distribution, décider entre binaires FFmpeg externes, bibliothèques dynamiques ou compilation dédiée. Documenter la configuration de compilation et la conformité de licence.

## Projet `.kito`

Spécification complète : `docs/KITO_FORMAT.md` (structure, temps en ticks, versions et migrations, conservation des données inconnues, sécurité déclarative, chemins portables, enregistrement atomique, sauvegarde automatique).

## Performance

- Les traitements lourds ne doivent jamais bloquer l'interface.
- Utiliser des tâches asynchrones et des canaux d'événements.
- Mettre en cache les miniatures, proxys et formes d'onde.
- Prévoir une politique de mémoire et de cache configurable.
