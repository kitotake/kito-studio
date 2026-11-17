# Format de projet `.kito` — version 1

Un projet est un **fichier JSON UTF-8** (`*.kito`), lisible et diffable. Les médias restent externes : le projet ne contient que leurs chemins. Code de référence : `src-tauri/src/project/` (`model.rs`, `validate.rs`, `migrate.rs`, `io.rs`, `media.rs`).

## Structure

```json
{
  "format": "kito-project",
  "formatVersion": 1,
  "minReaderVersion": 1,
  "savedBy": { "app": "Kito Studio", "version": "0.1.0" },
  "project": { "id": "uuid", "name": "Mon film", "createdAt": "2026-11-16T22:39:47Z", "modifiedAt": "…" },
  "settings": {
    "video": { "width": 1920, "height": 1080, "frameRate": { "num": 30, "den": 1 } },
    "audio": { "sampleRate": 48000 }
  },
  "media": [
    { "id": "m1", "kind": "video", "path": "D:/Rushes/a.mp4", "relativePath": "../../Rushes/a.mp4",
      "size": 123456, "modifiedAt": "2026-11-01T10:00:00Z" }
  ],
  "timeline": {
    "ticksPerSecond": 705600000,
    "sequences": [{
      "id": "…", "name": "Séquence 01",
      "tracks": [{
        "id": "…", "kind": "video", "name": "V1",
        "clips": [{
          "id": "…", "mediaId": "m1", "start": 0, "duration": 705600000, "sourceIn": 0,
          "effects": [{ "type": "blur", "params": { "radius": 4 } }],
          "transitions": [], "keyframes": []
        }]
      }]
    }]
  }
}
```

Un nouveau projet contient une séquence « Séquence 01 » avec 4 pistes vides (V1, V2, A1, A2).

## Temps

Toutes les positions et durées sont des **entiers** en « ticks », à raison de **705 600 000 par seconde**. Ce nombre se divise exactement par 24/25/30/50/60 images/s, 44,1/48/96 kHz et les cadences NTSC (24000/1001, 30000/1001, 60000/1001) : aucune dérive d'arrondi. Valeurs bornées à 2^53 − 1 (entier exact en JavaScript, soit environ 150 jours).

## Versions et migrations

- `formatVersion` : version d'écriture. `minReaderVersion` : plus ancienne version de Kito capable de lire le fichier sans risque (par défaut = `formatVersion`).
- **Plus récent mais compatible** (`minReaderVersion` ≤ version lue par l'application) : ouverture normale, `formatVersion` d'origine conservé à l'enregistrement, indication dans l'interface.
- **Plus récent et incompatible** : refus avec le code `INCOMPATIBLE_VERSION` ; le fichier n'est jamais modifié.
- **Plus ancien** : migration pas à pas (v1→v2→…) sur le JSON brut, avant le typage (`migrate.rs`, table `STEPS`). Avant le premier enregistrement d'un projet migré, l'ancienne version est copiée dans `<fichier>.kito.v<N>.bak` (jamais réécrite ensuite).
- Il n'existe que la version 1 : la table de migrations est vide. Le mécanisme est testé avec une table fictive (3 versions) ; un test garantit qu'une étape existe pour chaque version antérieure dès qu'on incrémente `CURRENT_FORMAT_VERSION`.

## Conservation des données inconnues

Chaque objet du modèle (racine, `savedBy`, `project`, `settings` et sous-objets, médias, séquences, pistes, clips) conserve ses champs inconnus et les réécrit tels quels ; les entrées d'effets/transitions/keyframes sont conservées en JSON brut. Un fichier produit par une version plus récente ne perd donc rien en passant par cette version. Les clés sont écrites dans un ordre déterministe (diffs lisibles).

## Sécurité : données déclaratives uniquement

Un `.kito` ne peut **jamais** déclencher l'exécution de code externe :

- Aucun champ n'est interprété comme un script, une commande, une URL ou un programme. Les chemins de médias sont seulement lus (`metadata`), jamais ouverts ni exécutés par le chargement du projet.
- Effets, transitions et keyframes doivent être des **objets** avec un champ `type` (identifiant strict : minuscules, chiffres, `_`, `.`, `-`, 64 caractères max). Une chaîne, un nombre, un objet sans `type` ou un `type` comme `../../x.exe` est refusé (`INVALID_FORMAT`).
- Imbrication JSON limitée à 16 niveaux ; fichier limité à 256 Mo ; identifiants uniques et références (`mediaId`) vérifiées ; bornes sur résolution, cadence, fréquence audio et temps.
- Le document vit côté Rust ; l'interface ne reçoit qu'un instantané (nom, réglages, état des médias), jamais le contenu brut des effets.
- Un champ comme `"command": "calc.exe"` dans un effet reste une simple donnée inerte : conservée, jamais exécutée (testé).
- Futur moteur de rendu : il ne devra interpréter que des types d'effets connus et ignorer les autres.

## Médias externes et portabilité

Chaque média stocke un chemin **absolu** (indicatif, tel que sur l'ordinateur d'origine) et un chemin **relatif au dossier du projet** (séparateur `/`, portable), plus sa taille. À l'ouverture, un média est cherché :

1. via le chemin relatif (survit au déplacement du projet avec ses médias, y compris vers un autre ordinateur ou une lettre de lecteur différente) ;
2. via le chemin absolu (survit au déplacement du projet seul).

Un candidat dont la taille correspond à celle enregistrée est préféré. Résultats : `ok`, `changed` (fichier présent mais de taille différente), `missing`. À chaque enregistrement (et « Enregistrer sous »), les chemins sont recalculés par rapport à la nouvelle position du projet, pour qu'il reste portable. L'interface propose de localiser un média à la main ou de chercher dans un dossier (liaison automatique uniquement si nom + taille correspondent à un seul fichier). Un lecteur différent ne permet pas de chemin relatif : seul le chemin absolu est alors utilisable.

## Enregistrement

- **Atomique** : écriture dans un fichier temporaire voisin → `fsync` → copie de l'ancienne version en `<fichier>.kito.bak` → renommage par-dessus la cible. En cas d'échec, le fichier d'origine est intact et le temporaire supprimé. Si le disque est plein ou l'accès refusé, l'erreur est explicite (`DISK_FULL`, `PERMISSION_DENIED`) et les modifications restent en mémoire.
- **Sauvegarde automatique** : copies horodatées dans `<dossier du projet>/Autosave/`, **jamais** dans le fichier principal. Réglages (Préférences) : activée ou non, intervalle (1, 2, 5, 10, 15, 30 min ; 5 par défaut), copies conservées (1–100 ; 10 par défaut). Elle ne s'exécute que si le projet a des modifications non enregistrées depuis la dernière copie.
- **Récupération après crash** : à l'ouverture, si la copie la plus récente est plus récente que le fichier, l'application propose de la restaurer. La restauration charge la copie comme travail **non enregistré** : le fichier principal ne change que par « Enregistrer » (qui garde alors la version précédente en `.bak`). Si le fichier principal est corrompu ou supprimé, la copie peut être restaurée depuis le message d'erreur d'ouverture. Choisir « Ne pas enregistrer » à la fermeture supprime les copies plus récentes pour qu'elles ne soient pas reproposées.
