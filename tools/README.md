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

## Téléchargement des sources

Le dossier `_sources/` est dans `.gitignore`. Pour regénérer le catalogue,
téléchargez d'abord les fichiers suivants depuis `tools/_sources/` :

```bash
cd _sources

# KANJIDIC2 (~15 Mo)
curl -sL "https://www.edrdg.org/kanjidic/kanjidic2.xml.gz" | gunzip -c > kanjidic2.xml

# KRADFILE + KRADFILE2 (~290 Ko)
curl -sL "https://raw.githubusercontent.com/tim-harding/Kradical/master/assets/edrdg_files/kradfile" -o kradfile
curl -sL "https://raw.githubusercontent.com/tim-harding/Kradical/master/assets/edrdg_files/kradfile2" -o kradfile2
# Convertir EUC-JP → UTF-8 (macOS)
iconv -f EUC-JP -t UTF-8 kradfile > kradfile_utf8 && mv kradfile_utf8 kradfile
iconv -f EUC-JP -t UTF-8 kradfile2 > kradfile2_utf8 && mv kradfile2_utf8 kradfile2

# JMdict English (~62 Mo, optionnel — pour générer les cards plus tard)
curl -sL "http://ftp.edrdg.org/pub/Nihongo/JMdict_e.gz" | gunzip -c > jmdict_e.xml

# Mapping JLPT kanji (Bluskyo, optionnel — version précédée déjà commitée)
curl -sL "https://github.com/Bluskyo/JLPT_Vocabulary/releases/latest/download/JLPT_kanji_ALL.json" -o jlpt_kanji.json
```

### Sources externes (non téléchargées dans le repo)

- **Japanese subtitle frequency list** — Chris Kempson.
  https://github.com/chriskempson/japanese-subtitles-word-kanji-frequency-lists

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
