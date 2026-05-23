# TODO / Roadmap

This document breaks down the development plan for **LearnWithAnime**. Tasks are grouped
by feature to allow different contributors and AIs to work in parallel.

## 1. Project Scaffolding
- [x] Create `frontend/` (Vue 3 + TypeScript) and `backend/` (Rust) workspaces.
- [x] Add `docker-compose.yml` for local development.
- [x] Set up CI workflow (GitHub Actions) running lint/tests for both layers.
- [x] Configure shared formatting tools: Prettier (frontend) & rustfmt/clippy (backend).

## 2. User Management & Onboarding
- [ ] Implement registration/login with JWT auth and password hashing.
- [ ] Store user profile (preferred manga series, known vocabulary, consent flags).
- [ ] Build onboarding questionnaire to collect initial manga knowledge.
- [ ] API endpoint to update user context after each session.

## 3. Flashcard Engine
- [ ] Design database schema for decks, cards, user progress, and scheduling.
- [ ] CRUD endpoints for cards and decks (admin/internal usage).
- [ ] Implement spaced‑repetition scheduler (SM‑2 variant or custom algorithm).
- [ ] Frontend card viewer supporting text, kanji, and audio playback.
- [ ] Allow users to answer via typing or voice; send results to AI evaluator.

## 4. AI Evaluation & Hint System
- [ ] Create microservice wrapping LLM API calls with per‑user context.
- [ ] Define prompt templates for answer checking and hint generation.
- [ ] Support hint tiers (e.g., first hint about radicals, second hint example word).
- [ ] Log AI interactions for analytics and future model tuning.

## 5. Audio & Speech Features
- [ ] Integrate TTS for sentence cards (cloud provider or open‑source models).
- [ ] Maintain library of pre‑recorded kanji readings for accuracy.
- [ ] Use Web Speech API (browser) and/or server‑side recognition for pronunciation
      evaluation.
- [ ] Provide playback controls and waveform display for recorded answers.

## 6. Progress Tracking & Gamification
- [ ] XP system tied to card difficulty and session streaks.
- [ ] Level tiers unlocking new manga‑themed quests or sentence translation mode.
- [ ] Achievement badges for milestones (e.g., "100 kanji mastered").
- [ ] Dashboard summarizing stats: accuracy, retention, favorite series, etc.

## 7. Deployment & Infrastructure
- [ ] Write production Dockerfiles for frontend and backend.
- [ ] Provision PostgreSQL & Redis instances via Docker or cloud services.
- [ ] Helm charts or Terraform modules for future Kubernetes/AWS deployment.
- [ ] Monitoring stack (Prometheus + Grafana) and log aggregation.

## 8. Future Enhancements
- [ ] Sentence translation challenges for advanced users.
- [ ] Community features: optional leaderboards, shared decks, manga discussion.
- [ ] Mobile app wrapper using Capacitor or React Native.
- [ ] Monetization layer (subscriptions, API usage metering).
- [ ] Data export/import compatible with Anki.

---
This roadmap is a living document. Update tasks as features evolve or new ideas emerge.
