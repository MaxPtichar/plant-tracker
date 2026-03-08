# 🌱 Plant Tracker

CLI tool for tracking indoor plant watering cycles using weight-based analytics and predictions.

## Idea

Plants are weighed before watering, after watering, and in between — by tracking weight changes the app calculates water consumption and predicts the next watering date.

## Features

- Add plants with species type
- Record weight measurements (before / after / between waterings)
- Full measurement history per plant
- Water consumption dynamics
- Next watering prediction
  - fewer than 3 waterings — uses baseline data per species
  - 3 or more waterings — calculated from real plant history

## Supported Species

- Chlorophytum
- Ficus Kinki
- Ficus Microcarpa
- Ficus Black Prince
- Sansevieria

## Tech Stack

- Rust
- Storage: JSON
- Dates: `chrono`

## Roadmap

- [✓] Telegram bot (async / Tokio / teloxide)
- [ ] Watering notifications
- [ ] Weight dynamics charts
- [ ] Add support for all spiecies
- [ ] Add per-user database (multi-user support)

## Status

In development — built as a Rust learning project.
