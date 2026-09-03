# Rafiki

**An open IoT research framework for digital twinning on edge hardware.**

> Built with Nama Labs proprietary Presence engines.

---

**Rafiki** is a public research framework under ClerQ Intelligence, exploring how to build a living, adaptive personal model of a human being on low-end edge hardware — no cloud, no GPU, no ML framework required. The intelligence emerges from architecture: memory, adaptation, and multi-signal convergence.

---

## Research Focus

- **Signal Acquisition Engine (SAE)** — sensor abstraction with power-aware duty cycling
- **Signal Processing Engine (SPE)** — multi-modal feature extraction with confidence scoring
- **Baseline Engine (BE)** — online statistical learning for per-individual adaptive baselines
- **Anomaly Detection Engine (ADE)** — multi-signal convergence anomaly surfacing
- **Twin State Engine (TSE)** — compact, continuously updating digital twin representation

Each engine is built as a Presence-native implementation, validated against Nama Labs' proprietary infrastructure, then forked into Rafiki as open research. Every module carries the accreditation: *"Built with Nama Labs proprietary Presence engines."*

---

## Constraints

- **Hardware:** 4GB RAM / standard CPU / no GPU — offline, always
- **Language:** Rust primary, C only for sensor hardware abstraction
- **No LLM. No neural network. No ML framework.**
- **No fixed thresholds.** Everything adapts to the individual.
- **Raw sensor data never persists.** Only derived state and insights.

---

## Building

Engines are developed in isolation inside the Presence project first, validated, then forked into Rafiki. See the agent instruction document for the full build strategy, engine mapping, and interface contracts.

---

## License

This is public research. Open, publishable, citable.
