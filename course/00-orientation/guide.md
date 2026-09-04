# Module 00: Orientation (15 minutes)

Goal: leave with a running pipeline on your own machine and know
where everything in this course lives.

## Words first

- **Repository (repo):** a project folder tracked by git, usually
  hosted on GitHub. Ours: `ClerQ-Intelligence-CQ/Rafiki`.
- **Crate:** Rust's word for a library or program package. Rafiki is
  a workspace of crates, one per engine plus the assembly wiring.
- **Release build:** the optimized binary (`--release`). Our
  benchmarks always use it; debug builds are slower and prove nothing.
- **Offline-first:** everything here runs with no network after the
  one-time toolchain install. If a step needs the network, it says so.

## Setup

1. Install Rust via https://rustup.rs (accept defaults).
2. Check: `cargo --version` and `rustc --version` both answer.
3. Clone: `git clone https://github.com/ClerQ-Intelligence-CQ/Rafiki`
4. Enter: `cd Rafiki`
5. First run: `cargo run --release -p rafiki-sae --example simulate`

Expected: an init line, typed sensor events with confidence scores,
a duty-cycle drop to 500ms on quiet input, sample counts, and a
memory line. If you see that, your machine is a lab now.

## Map of the course

Each module folder holds `guide.md` (the lesson), an interactive
`.html` page, and an `experiment/` folder with its own README,
scripts, and expected outputs. The zips in each folder are the same
experiments packed for sharing.

## Troubleshooting

- `cargo` not found: close and reopen the terminal after installing
  Rust (PATH updates on new shells).
- Slow first build: normal. Dependencies compile once, then cache.
- Anything else: open an issue on the repo with your OS, the exact
  command, and the last 20 lines of output.

Next: [Module 01](../01-first-principles/guide.md).
