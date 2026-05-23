# LearnWithAnime

LearnWithAnime is a long‑term project aimed at turning the passive Japanese vocabulary
picked up from watching anime into active language skills. The app mixes
AI‑assisted flashcards, adaptive practice, and gamified progression so that users can
train kanji recognition, vocabulary usage, listening, and pronunciation.

## Project Goals

* **Activate existing knowledge:** Help fans convert words and phrases they have heard
  repeatedly in VOSTFR/VOSTEN media into usable Japanese.
* **Personalized learning:** AI agents evaluate answers, provide hints, and adjust
  difficulty based on user performance and declared anime preferences.
* **Cross‑platform availability:** Responsive web app (desktop & mobile) with a cloud
  backend scalable from a single VPS to Kubernetes.

## High‑Level Architecture

| Layer      | Technology (initial suggestion)                                |
| ---------- | -------------------------------------------------------------- |
| Frontend   | Vue 3 + TypeScript + Vite, PWA, Web Speech API                 |
| Backend    | Rust (Axum or Actix‑web), GraphQL/REST APIs                    |
| Database   | PostgreSQL (core data) + Redis (sessions/cache)                |
| AI Service | OpenAI or compatible LLMs wrapped by a dedicated microservice  |
| Container  | Docker for dev/prod parity, deployable to K8s or bare VPS      |

## Core Modules

1. **Flashcard Engine** – spaced repetition, kanji & vocabulary decks, audio playback.
2. **AI Evaluation** – natural‑language feedback, hint generation, difficulty tuning.
3. **Speech & Audio** – TTS for sentences, optional pre‑recorded kanji readings,
   speech recognition for user answers.
4. **Progress & Gamification** – XP, levels, streaks, achievements, daily quests.
5. **Content Pipeline** – onboarding questionnaire, per‑user context store, anime
   references used as thematic anchors for vocabulary.

## Repository Layout (planned)

```
LearnWithAnime/
├── backend/        # Rust API services
├── frontend/       # Vue application
├── infra/          # Docker, CI/CD, deployment manifests
├── docs/           # Architecture notes and design decisions
└── TODO.md         # Feature roadmap and task breakdown
```

## Contributing

The project is at the planning stage. See `TODO.md` for the current roadmap.
Feel free to suggest improvements or alternative approaches.

