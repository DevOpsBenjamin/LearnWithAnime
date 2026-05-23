# LearnWithAnime — Outils de génération du catalogue

## Attributions

### KANJIDIC2 et KRADFILE/RADKFILE

Copyright © 2025 Electronic Dictionary Research and Development Group (EDRDG).
Utilisés sous licence **CC-BY-SA 4.0**.

- **KANJIDIC2** — Fichier XML complet des ~13 000 kanji JIS avec lectures,
  significations, niveaux JLPT, fréquences, traits et radicaux.
  https://www.edrdg.org/kanjidic/kanjidic2.xml.gz
- **KRADFILE / RADKFILE** — Copyright © 2001–2007 Michael Raine,
  James Breen et EDRDG. Décompositions kanji ↔ composants visuels.
  https://www.edrdg.org/krad/kradinf.html

Voir https://www.edrdg.org/edrdg/licence.html pour les conditions complètes.

### JMdict

Fichier XML complet des ~170 000 entrées lexicales japonaises avec traductions
en anglais. Distribué par EDRDG depuis ftp.edrdg.org.
https://www.edrdg.org/jmddict/j_jmdict.html

### jlpt_kanji.json

Les données de mapping kanji → niveau JLPT (N5→N1, nouveau système 2010)
proviennent de [Bluskyo/JLPT_Vocabulary](https://github.com/Bluskyo/JLPT_Vocabulary)
(MIT), elles-mêmes basées sur les listes de [tanos.co.uk](https://www.tanos.co.uk/jlpt/)
(CC-BY, Jonathan Waller).

### Japanese subtitle frequency list

Fréquences de mots depuis 12 277 sous-titres japonais (anime, drama, films).
Projet de Chris Kempson. https://github.com/chriskempson/japanese-subtitles-word-kanji-frequency-lists

## Sources téléchargées mais pas encore traitées

Les fichiers suivants sont présents dans `_sources/` mais aucun script ne les
exploite encore :

- `jmdict_e.xml` — dictionnaire lexical japonais-anglais (pour générer les cards)
- `radkfile` / `radkfile2` — inverse de KRADFILE (radical → kanji)
- `kradfile2` — décompositions pour kanji JIS X 0212 étendus (déjà mergé par enrich-catalog)

La liste de fréquences de Chris Kempson n'est pas téléchargée dans le repo ;
suivre le lien GitHub pour l'obtenir si nécessaire.

## Procédure de regénération

Les fichiers `backend/data/kanji/jlpt-n*.jsonl` sont générés à partir des sources
ci-dessus. Pour les regénérer :

```bash
cargo run --bin enrich-catalog
```

Le script lit `_sources/`, enrichit les kanji listés dans `jlpt_kanji.json` avec
les données KANJIDIC2 et KRADFILE, puis écrit les 5 fichiers JSONL dans
`backend/data/kanji/`.

En cas d'erreur (kanji non trouvé dans KANJIDIC2, ambiguïté, etc.), le script
affiche un rapport et ne produit pas de fichiers invalides.
