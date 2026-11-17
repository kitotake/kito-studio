# Tests

## Lancer

```powershell
pnpm install
pnpm test             # React : Vitest + Testing Library (jsdom)
pnpm build            # TypeScript strict + Vite
cd src-tauri
cargo test            # Rust : unitaires + intégration
cargo fmt --check
```

## Ce qui est couvert

**Rust — unitaires** (`src/**` modules `tests`) : erreurs et codes, horodatages, noms de projet, chemins relatifs/absolus, modèle et conservation des champs inconnus, validation (dont données déclaratives), migrations (table fictive), lecture/écriture atomique (échec injecté, fichier tronqué, BOM, taille max), résolution et reliage des médias, sauvegarde automatique (échéance, rotation, récupération), récents, préférences, session (création, modification, enregistrer, enregistrer sous, récupération, reliage).

**Rust — intégration** (`src-tauri/tests/project_lifecycle.rs`) : cycle de vie complet sur de vrais fichiers, projet déplacé sur « un autre ordinateur », fichier corrompu/tronqué, versions, crash + récupération sans écraser le fichier manuel, sauvetage quand le fichier principal est corrompu ou supprimé, abandon volontaire des modifications, données inconnues sur toute la chaîne, contenu hostile inerte, effets non déclaratifs refusés, persistance des préférences et récents.

**React** : erreurs (`KitoError`, messages compréhensibles), client Tauri, validation de nom, formulaire de paramètres, formats, hooks (modifications non enregistrées, raccourcis, fermeture de fenêtre), composants (création, récents, médias manquants, état de sauvegarde, `ErrorBoundary`) et **flux complets de l'application** avec l'API native simulée (création, ouverture, navigation, enregistrement et échecs d'écriture, récupération, médias manquants, paramètres, préférences).

**Test de mutation manuel** : 6 comportements critiques ont été cassés volontairement (chemins relatifs, nettoyage du temporaire, effets non déclaratifs, récupération, garde du dossier Autosave, copie `.bak`) ; la suite Rust a échoué à chaque fois.

**Essais de bout en bout dans l'application native** (Linux, écran virtuel) : création d'un vrai projet, écriture du `.kito`, récents, sauvegarde automatique réelle, arrêt brutal puis récupération, enregistrement avec `.bak`, interception de la fermeture de fenêtre avec modifications non enregistrées (« Ne pas enregistrer » ferme l'application et supprime la copie abandonnée), ouverture avec média manquant, ouverture du sélecteur de dossier natif.

## Ce qui n'est PAS testé

- **Windows** : aucun test n'a été exécuté sous Windows (ni `pnpm tauri dev`, ni l'installateur). Les chemins Windows (lettres de lecteur, noms réservés, `\`) sont couverts par des tests conditionnels `cfg(windows)` qui n'ont pas pu être exécutés.
- Les sélecteurs de fichiers natifs : leur ouverture a été vérifiée, mais la validation d'un choix n'a pas pu être automatisée (le sélecteur GTK ne navigue pas sans dbus). La logique appelée ensuite est testée avec un sélecteur simulé.
- Tests de charge (projets très volumineux), disques réseau et supports amovibles.
- `clippy` n'était pas disponible dans l'environnement de test.
