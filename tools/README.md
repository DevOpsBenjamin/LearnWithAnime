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

### Sous-titres anime — kitsunekko-mirror

Sources de sous-titres japonais pour l'extraction de fréquences par anime.
~3400 séries + 950 films. Clone uniquement l'historique récent (shallow).

```bash
cd _sources
git clone --depth 1 https://github.com/Ajatt-Tools/kitsunekko-mirror.git kitsunekko
```

Projet original : https://kitsunekko.net/ — miroir maintenu par Ajatt-Tools.

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

# Sous-titres anime — kitsunekko-mirror (~3 Go)
git clone --depth 1 https://github.com/Ajatt-Tools/kitsunekko-mirror.git kitsunekko
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

## Extraction de fréquences par anime

### Anime simple

```bash
# Anime individuel
cargo run --bin extract-anime-freq -- "_sources/kitsunekko/subtitles/anime_tv/Sousou no Frieren" --min-count 5
```

Le résultat est écrit dans `tools/data/anime_freq/<slug>.jsonl`.

### Fusion de franchises multi-dossiers

Certaines séries sont réparties dans plusieurs dossiers (Naruto + Shippuden,
Dragon Ball + Z + Super, Pocket Monsters XY + Sun & Moon + (2023), etc.).
Pour les merger en un seul fichier de fréquences :

1. Éditer `merge_groups.toml` pour définir ou ajuster les groupes.
2. Lancer le script :

```bash
./merge_and_extract.sh
```

Le script :
- Lit `merge_groups.toml`
- Symlinke tous les `.srt`/`.ass` des dossiers source vers `_sources/merged/<Groupe>/`
- Calcule le `--min-count` adaptatif selon le nombre total de fichiers
- Lance `extract-anime-freq` sur le dossier merger (support merge si re-run)
- Nettoie les symlinks (le fichier `.jsonl` est conservé dans `data/anime_freq/`)

Les noms de dossiers dans le TOML doivent correspondre exactement aux noms
dans `_sources/kitsunekko/subtitles/anime_tv/`.
