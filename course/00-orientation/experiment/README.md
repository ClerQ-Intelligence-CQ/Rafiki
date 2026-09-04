# Module 00 experiment: env-check

## Objective

Prove the machine is a lab: toolchain present, repo builds, simulator runs.

## Steps

1. `cargo --version` (expect a version number, e.g. 1.8x).
2. `rustc --version` (same).
3. `cargo build -p rafiki-sae` (expect success, warnings at most).
4. `cargo run --release -p rafiki-sae --example simulate` (expect the
   init line, events, duty drop, counts, memory line).

## Expected outputs

- Step 1-2: two version strings, no errors.
- Step 3: `Finished` with no `error` lines.
- Step 4: output ending in sample counts and a memory line.

## Troubleshooting

See `guide.md`. If step 4 fails but step 3 passed, rerun with
`--release` spelled exactly (debug works too, just slower).

## Scripts

`env_check.sh` (POSIX) and `env_check.ps1` (Windows) run all four
steps and stop at the first failure.
