<div align="center">

<h1>
  <img src="https://img.shields.io/badge/🦜-Rafiki-FF6B35?style=for-the-badge&logoColor=white" alt="Rafiki">
  <br>
  <span style="font-size: 1.4em; color: #333;">An Open IoT Research Framework for Digital Twinning on Edge Hardware</span>
</h1>

[![License: MIT](https://img.shields.io/badge/License-MIT-%23333?style=for-the-badge&logo=opensourceinitiative&logoColor=white)](https://opensource.org/licenses/MIT)
[![Built with Nama Labs proprietary Presence engines](https://img.shields.io/badge/%F0%9F%8E%99-Built%20with%20Nama%20Labs%20proprietary%20Presence%20engines-%23E63946?style=for-the-badge&logo=flame&logoColor=white)](https://github.com/nama-labs/presence)
[![Target: 4GB RAM / No GPU](https://img.shields.io/badge/%E2%9A%A1-4GB%20RAM%20/%20No%20GPU%20/%20Offline%20first-%232A9D8F?style=for-the-badge&logo=cpu&logoColor=white)](https://github.com/ClerQ-Intelligence-CQ/Rafiki)
[![Language: Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Status: Research in Progress](https://img.shields.io/badge/%F0%9F%94%AC-Research%20in%20Progress-%23E76F51?style=for-the-badge)](https://github.com/ClerQ-Intelligence-CQ/Rafiki)

</div>

<div align="center">

> *"How do you build a living, adaptive personal model of a human being on a $30 device with no internet, no GPU, and no machine learning framework — and make it feel genuinely intelligent without a single hardcoded rule?"*

</div>

---

## 🧬 What Is Rafiki?

Rafiki is a **public IoT research framework** for high-efficiency **digital twinning of a human on low-end edge hardware**. It lives under [ClerQ Intelligence](https://github.com/ClerQ-Intelligence-CQ), co-owned with [Nama Research Labs](https://github.com/nama-labs).

The intelligence here **emerges from architecture** — memory, adaptation, and multi-signal convergence — not from rules or models. The system learns *you* specifically. No population thresholds. No hardcoded "cough twice = alert." Just: *"this person's pattern deviated from their own baseline in a way that warrants surfacing."*

<div align="center">

```
┌─────────────────────────────────────────────────────────┐
│                    THE RAFIKI STACK                      │
│                                                         │
│   ┌──────────┐  ┌──────────┐  ┌──────────┐             │
│   │   SAE    │  │   SPE    │  │    BE    │             │
│   │  Signal  │→ │  Signal  │→ │  Baseline│             │
│   │ Acquisition│→│Processing│→ │          │             │
│   └──────────┘  └──────────┘  └──────────┘             │
│        ↓              ↓              ↓                  │
│   ┌──────────────────────────────────────┐             │
│   │        ADE  │  TSE                   │             │
│   │  Anomaly   │  Twin State             │             │
│   │  Detection │  Engine                 │             │
│   └──────────────────────────────────────┘             │
│                                                         │
│   ═══ Presence (proprietary) ═══                        │
│   Bezaliel → Penemue → Baraqiel                         │
└─────────────────────────────────────────────────────────┘
```

</div>

---

## 🏗️ Build Strategy — Presence First, Fork After

Every Rafiki engine is **built inside the Presence project first** as a native implementation of the corresponding Presence engine primitive. Once validated against Presence's four-layer architecture (domain primitive layer, temporal layer, event bus, MCP exposure), it's forked into Rafiki as the **public open research expression** of that work.

The full instruction document lives in the **[Fiti-with-Nama](https://github.com/ClerQ-Intelligence-CQ/Fiti-with-Nama)** private repo (the instruction doc is there — Rafiki knows nothing about Fiti, that's Fiti's concern).

<div align="center">

| Step | Action |
|------|--------|
| 1️⃣ | Identify the corresponding Presence engine |
| 2️⃣ | Build as a Presence-native implementation |
| 3️⃣ | Validate & benchmark inside Presence |
| 4️⃣ | Fork into Rafiki, strip proprietary coupling |
| 5️⃣ | Document the interface boundary |
| 🔥 | **Accreditation: "Built with Nama Labs proprietary Presence engines"** |

</div>

---

## 🔥 The Five Engines

Each engine is built in isolation. Input contract in, output contract out. Knows nothing about the others until assembly time.

| Engine | Presence Origin | What It Does | Status |
|--------|----------------|-------------|--------|
| <span style="color:#E63946;">**SAE**</span> Signal Acquisition | Bezaliel / Earth — <code>Soileater</code> subengine | Sensor abstraction layer. Raw hardware input → typed event stream. Power-aware duty cycling. <em>Never stores content — only amplitude envelopes.</em> | 🔵 <b>NEXT UP</b> |
| <span style="color:#2A9D8F;">**SPE**</span> Signal Processing | Bezaliel / Earth + Penemue / Life | Feature extraction per signal type. Sleep/wake inference, noise exposure scoring, terrain classification. | ⚪ Pending |
| <span style="color:#E76F51;">**BE**</span> Baseline Engine | Penemue / Life — <code>Sentinel</code> subengine | The heart of Rafiki. Online statistical learning, per-individual adaptive baselines. Solves the cold-start problem from statistical properties of data. | ⚪ Pending |
| <span style="color:#264653;">**ADE**</span> Anomaly Detection | Baraqiel / Fire — <code>Aegis</code> subengine | Multi-signal convergence anomaly surfacing. Single-signal deviation = noise. Multi-signal convergence = signal worth surfacing. | ⚪ Pending |
| <span style="color:#606c38;">**TSE**</span> Twin State Engine | Penemue / Life | The living compact representation. Kilobytes, not megabytes. Continuously updated digital twin. Export interface for Penemue/Raphael. | ⚪ Pending |

---

## 🔗 Nama Integration Interfaces

Clean, documented, unimplemented — just the contracts:

| Boundary | Rafiki Side | Nama Side |
|----------|------------|-----------|
| 🧬 Living state export | <span style="color:#E76F51;">TSE</span> export interface | Penemue/Raphael (Life engine) |
| 🔒 Consent gate | <span style="color:#E63946;">SAE</span> consent gate interface | Armaros/Raguel (Law engine) — <code>Covenant</code> subengine |
| 🗺️ Spatial swap boundary | <span style="color:#2A9D8F;">SPE</span> local terrain classification | Bezaliel/Saraqael (Earth engine) — richer spatial indexing |

---

## ⚡ Hard Constraints

<div align="center">

| Constraint | Policy |
|-----------|--------|
| 🧠 LLM / Neural Network / ML | <span style="color:#E63946; font-weight:bold;">❌ NONE.</span> Not a single one. |
| ☁️ Cloud dependency | <span style="color:#E63946; font-weight:bold;">❌ OFFLINE-FIRST.</span> Always. |
| 📏 Fixed thresholds | <span style="color:#E63946; font-weight:bold;">❌ NEVER.</span> Everything adapts to the individual. |
| 💾 Raw sensor data persistence | <span style="color:#E63946; font-weight:bold;">❌ NEVER.</span> Only derived state and insights survive. |
| ⚡ Power budget | <span style="color:#E63946; font-weight:bold;">FIRST-CLASS</span> — not an afterthought. |
| 🖥️ Language | Rust primary. C only for sensor hardware abstraction. |
| 📐 Target bench | <span style="color:#2A9D8F; font-weight:bold;">4GB RAM / standard CPU / no GPU.</span> Benchmark against this constantly. |

</div>

---

## 📚 Research Output Standard

Every engine ships with:

- ✅ Clean Rust implementation with documented design rationale
- ✅ Benchmark results on the 4GB / no-GPU target
- ✅ Memory profile (peak + steady-state)
- ✅ Power consumption estimate (where measurable)
- ✅ Known limitations and open questions
- ✅ Clear input/output interface contract

<em>This is public research. It must be readable and useful to someone who was not in the room when it was designed.</em>

---

## 🧊 The Cold Start Problem

Open research question: The Baseline Engine has no data on day one. The system cannot detect anomalies until it has a stable personal baseline. What is the mathematically principled minimum data collection period before the BE produces trustworthy output? What does the system surface to the user during that period?

<em>Do not hardcode a number. Derive it from the statistical properties of the data.</em>

---

## 📁 Planned Repo Structure

<details>
<summary><b>Click to expand — for reference, not created yet</b></summary>

```
rafiki/
  engines/
    sae/        🔵 Signal Acquisition (NEXT UP)
    spe/        🟢 Signal Processing
    be/         🟠 Baseline Engine
    ade/        🔵 Anomaly Detection
    tse/        🟢 Twin State Engine
  benchmarks/   📊 Results, profiles, power estimates
  docs/
    design/     📝 Rationale per engine
    interfaces/ 🔗 Nama integration contracts
  research/     📄 Papers, findings, open questions
```

</details>

---

## 🚀 Start Here

**The first thing to build is the Signal Acquisition Engine (SAE)** — inside Bezaliel / Earth, as the <code>Soileater</code> subengine implementation.

- Rust, sensor interface must be real, data source can be simulated
- Benchmark memory and CPU on the target spec (4GB / no GPU)
- Document every design decision
- When SAE is validated, report back and we move to SPE

---

## 🤝 Built With

<div align="center">

[<img src="https://img.shields.io/badge/Nama%20Labs-%20AI%20Research%20Company-%23E63946?style=for-the-badge&logo=flame&logoColor=white)](https://github.com/nama-labs)
[<img src="https://img.shields.io/badge/Presence-%20Proprietary%20AI%20Infrastructure-%23264653?style=for-the-badge&logo=server&logoColor=white)](https://github.com/nama-labs/presence)
[<img src="https://img.shields.io/badge/ClerQ%20Intelligence-%20Clinical%20Venture-%232A9D8F?style=for-the-badge&logo=shield&logoColor=white)](https://github.com/ClerQ-Intelligence-CQ)
[<img src="https://img.shields.io/badge/Rust-%20Programming%20Language-%23000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)

</div>

<p align="center">

<sub>
<b>Every module in Rafiki is accredited:</b>
<mark style="background:#E63946;color:#fff;padding:2px 8px;border-radius:3px;">"Built with Nama Labs proprietary Presence engines"</mark>
</sub>

</p>

<p align="center">
<i>Rafiki — open research, publishable, citable. The world's first edge-native digital twinning framework.</i>
</p>
