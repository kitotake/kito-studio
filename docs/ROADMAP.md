# Roadmap

## Phase 0 — Fondations
- [x] Shell React / TypeScript / SCSS
- [x] Configuration Tauri 2
- [x] Backend Rust minimal
- [x] Diagnostic FFmpeg / FFprobe
- [x] Maquette accueil et espace de montage
- [x] Build frontend + backend validés sous Linux (voir `docs/AUDIT.md`)
- [x] Architecture interne : découpage React, SCSS par couche, erreurs structurées Rust/React
- [ ] Premier build Windows validé (`pnpm tauri dev` puis `pnpm tauri build`)

## Phase 1 — Projet et médias
- [x] Modèle de projet versionné (`docs/KITO_FORMAT.md`)
- [x] Création, ouverture et sauvegarde `.kito` (atomique, `.bak`)
- [x] Sauvegarde automatique configurable et récupération après crash
- [x] Projets récents, indicateur de sauvegarde, modifications non enregistrées
- [x] Chemins de médias portables et reliage des médias
- [ ] Boîte de dialogue d'importation
- [ ] Probe des médias avec FFprobe
- [ ] Miniatures
- [x] Détection des fichiers manquants (le modèle les référence ; l'import les créera)

## Phase 2 — Timeline
- [ ] Séquences et pistes
- [ ] Clips et opérations temporelles
- [ ] Undo / redo
- [ ] Lecture simple
- [ ] Découpage et déplacement

## Phase 3 — Moteur média
- [ ] Décodage
- [ ] Prévisualisation
- [ ] Audio synchronisé
- [ ] Proxys
- [ ] Composition
- [ ] Export H.264

## Phase 4 — Édition avancée
- [ ] Effets
- [ ] Transitions
- [ ] Keyframes
- [ ] Titres
- [ ] Mixage
- [ ] Colorimétrie

## Phase 5 — Android
- [ ] Initialisation Android Tauri
- [ ] Adaptateur média mobile
- [ ] UI tactile
- [ ] Tests de compatibilité de projet

## Phase 6 — Cloud et fonctions avancées
- [ ] Synchronisation optionnelle
- [ ] Conflits et historique
- [ ] IA facultative
- [ ] Extensions
