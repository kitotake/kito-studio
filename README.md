# Kito Studio

Prototype initial d'un logiciel de montage vidéo multiplateforme.

## État actuel

**Implémenté et testé (sous Linux ; Windows à valider)** :

- Création d'un projet : nom, emplacement, résolution (720p, 1080p, 1440p, 4K, vertical), fréquence d'images (24, 25, 30, 50, 60), fréquence audio (44,1, 48, 96 kHz).
- Ouverture, enregistrement (atomique, avec copie `.bak`) et « Enregistrer sous » au format `.kito` versionné (`docs/KITO_FORMAT.md`).
- Sauvegarde automatique configurable (Préférences) et récupération après un arrêt brutal, sans jamais écraser le dernier enregistrement.
- Projets récents, indicateur de sauvegarde, confirmation avant de perdre des modifications (y compris à la fermeture de la fenêtre).
- Détection des médias manquants ou modifiés, chemins relatifs portables d'un ordinateur à l'autre, reliage manuel ou par dossier.
- Détection de `ffmpeg` / `ffprobe`.
- Raccourcis : Ctrl+N (nouveau), Ctrl+O (ouvrir), Ctrl+S (enregistrer), Ctrl+Maj+S (enregistrer sous).

**Pas encore fonctionnel** : importation de médias, timeline éditable, lecture, effets, audio, rendu et export. La timeline et le moniteur de l'éditeur sont des emplacements statiques ; le projet les stocke déjà (séquences, pistes, clips, effets, transitions, keyframes) mais l'interface ne les édite pas.

Où sont les données : un projet est un dossier `<emplacement>/<nom>/` contenant `<nom>.kito`, éventuellement `<nom>.kito.bak` et `Autosave/`. Les préférences et la liste des récents sont dans le dossier de données de l'application (`%APPDATA%\com.kitotake.kitostudio` sous Windows).

## Prérequis Windows

- Windows 10/11.
- Node.js LTS.
- pnpm 10 ou supérieur.
- Rust stable avec toolchain MSVC.
- Microsoft C++ Build Tools avec « Desktop development with C++ ».
- WebView2 Runtime.

Voir les prérequis officiels Tauri : https://v2.tauri.app/start/prerequisites/

## Installation

Depuis PowerShell, dans le dossier du projet :

```powershell
corepack enable
pnpm install
```

## Lancer en mode navigateur

```powershell
pnpm dev
```

Ouvrir http://localhost:1420.

Le mode navigateur permet de travailler sur l'interface, mais les commandes Rust ne sont disponibles que dans l'application Tauri.

## Lancer l'application Windows

```powershell
pnpm tauri dev
```

## Construire l'installateur Windows

```powershell
pnpm tauri build
```

L'installateur NSIS sera généré dans `src-tauri/target/release/bundle/nsis/`.

## Vérifier les outils média

Installer une distribution FFmpeg compatible et rendre `ffmpeg.exe` et `ffprobe.exe` accessibles dans le `PATH`.

Dans PowerShell :

```powershell
where.exe ffmpeg
where.exe ffprobe
ffmpeg -version
ffprobe -version
```

L'application détecte leur présence, mais ne lance pas encore de transcodage.

## Android — étape ultérieure

Le même projet Tauri sera initialisé pour Android après validation de la version Windows :

```powershell
pnpm tauri android init
pnpm tauri android dev
```

Pré-requis supplémentaires : Android Studio, Android SDK, JDK et configuration des variables d'environnement. Voir https://v2.tauri.app/start/prerequisites/

## Vérifier le projet

```powershell
pnpm build            # TypeScript + Vite
pnpm test             # tests React (Vitest)
cd src-tauri
cargo check           # backend Rust
cargo test            # tests Rust (unitaires + intégration)
cargo fmt --check     # formatage Rust
```

Voir `docs/TESTING.md` pour ce qui est couvert et ce qui ne l'est pas.

## Structure

```text
src/
  app/           Assemblage (App, ErrorBoundary, état de navigation)
  components/    Briques UI génériques et layout
  features/      Une fonctionnalité par dossier (home, project, editor, system)
  lib/           Utilitaires transverses (erreurs, client Tauri)
  types/         Types partagés
  styles/        SCSS par couche (abstracts, base, layout, components, features)
src-tauri/       Application native Rust/Tauri (commands/, error.rs, modules métier)
branding/        Sources des icônes (régénérer avec `pnpm tauri icon branding/app-icon.png`)
docs/            Architecture, format .kito, décisions, roadmap, tests, audit
```

Détail des couches et règles de dépendance : `docs/ARCHITECTURE.md`.

## Règles de contribution

- Garder `strict` activé dans TypeScript.
- Pas de Tailwind.
- Pas de logique de montage dans les composants React.
- Les composants n'appellent jamais `invoke` : passer par `nativeApi` (`src/lib/tauri`).
- Toute commande Rust retourne `AppResult<T>`.
- Toute opération média lourde doit être exécutée hors du thread UI.
- Toute donnée de projet persistante doit être versionnée.
- Documenter les décisions architecturales importantes.
- Ne jamais modifier les médias originaux.

## FFmpeg et licences

FFmpeg peut être distribué sous LGPL ou GPL selon les composants et options de compilation utilisés. Avant toute redistribution, figer la configuration, inventorier les codecs et bibliothèques externes et respecter les obligations correspondantes.

Référence officielle : https://www.ffmpeg.org/legal.html
