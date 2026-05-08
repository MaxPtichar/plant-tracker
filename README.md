# 🌱 Plant Tracker
Indoor plant watering tracker with weight-based analytics. Telegram bot with watering predictions and measurement logging.

## Idea
Plants are weighed before watering, after watering, and in between — by tracking weight changes the app calculates water consumption and predicts the next watering date.

## Features
- Add plants with species type
- Record weight measurements (before / after / between waterings)
- Full measurement history per plant
- Water consumption dynamics
- Next watering prediction
  - no history, no measurements — uses Penman-Monteith physical model
  - watering history available — blended from historical cycles
  - mid-cycle measurements available — prioritises current cycle data
- Multi-user support — each Telegram user has their own isolated plant database


## Tech Stack
- Rust
- Storage: PostgreSQL (`sqlx`)
- Telegram: `teloxide` (async / Tokio)
- Dates: `chrono`

## Roadmap
- [✓] Telegram bot (async / Tokio / teloxide)
- [✓] Watering notifications
- [✓] Add support for all species
- [✓] Add per-user database (multi-user support)
- [ ] Add localization (EN / RU)

## Status
In development — built as a Rust learning project.
