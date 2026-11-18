# Validation

Two ladders: the original one at the retired `.hnmod` format, and the SDK-format one after
the 2026-11-18 migration. Logs: `../_shared/evidence/imfast/`.

## SDK format (2026-11-18) — the current ladder

Run on 2026-11-18 in `ports/imfast` at nightly
`0b14f8acfd97fa2c750e42795249dcc981178ca4` / mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0` (launcher 0.2.78), Rust 1.97.0,
`CARGO_INCREMENTAL=0`. The commands run **directly in the port** (`cargo horizon-mod`), not
through `build-slot.sh`, which still speaks the retired tool; there is no staging step.

| Step | Command | Result |
|---|---|---|
| Compile + pure tests | `cargo test` | **5 passed**, 0 failed (`sdk-test.log`) |
| Formatting | `cargo fmt --check` (not `--all`: that would check the fetched SDK) | clean (`sdk-fmt.log`) |
| Lints | `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build` | `target/horizon-mod/imfast.hmod`, exit 0 (`sdk-build.log`) |
| Registry check | `cargo horizon-mod check imfast.hmod` | `imfast 0.2.0 (native-mod, format native-1/0b14f8a/e59da90, for 26.3)`, 7,570 B, **exit 0** (`sdk-check.log`, `sdk-check-json.log`) |
| What it uses | `cargo horizon-mod check imfast.hmod --uses` | 24 surface entries (`sdk-uses.json`), exit 0 (`sdk-check-uses.log`) |
| Fits the build | `cargo horizon-mod fits imfast.hmod --build nightly` | `0b14f8a made for`, exit 0 (`sdk-fits.log`) |
| Package identity | `sha256sum imfast.hmod` | `f9bd961c261253f6c0f0502911b1f2bf91f1052f8332abab22fbce83d83f7510`, 8 entries (`sdk-sha256.txt`) |
| `runtime.rs` untouched | `unzip -p imfast.hmod src/runtime.rs \| sha256sum` | `f86b58a9f3b0d7ac013b95a1ed451350e018ad0f78edcb5756e5dfbd1e7719b8`, identical to the retired file |

**Not re-run: the in-game e2e.** It needs a game the launcher built with the mod
(`horizon-launcher native-build … --trust-native-code`), and `horizon-launcher` is not
installed here. Run it with `GAME=<launcher-built horizon-client> tools/e2e.sh` and record
`PASS` + `EXIT:0` before moving this port to `ported`. The behaviour itself is unchanged:
`src/runtime.rs` is byte-identical and the glue runs the same two console lines.

## The retired `.hnmod` ladder (2026-11-09, kept as history)

Ladder run on 2026-11-09 at the campaign pin (Horizon
`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`, horizon-mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57`, schema 6), on Linux with Rust 1.95.0,
every step inside a `ports/tools/build-slot.sh` lease with `CARGO_INCREMENTAL=0`.

| Step | Command | Result |
|---|---|---|
| Runtime tests | `cargo test --locked --offline` | 5 passed, 0 failed (`runtime-test.log`) |
| Formatting | `cargo fmt --all --check` | clean |
| Lints | `cargo clippy --all-targets --locked --offline -- -D warnings` | exit 0, 0 warnings (`clippy.log`) |
| Stage | `python3 tools/horizon.py stage` | exit 0; a re-run reports `stage_changed: false` (`horizon-stage.json`) |
| Staged check | `python3 tools/horizon.py check` | `cargo_exit: 0`, 0 warnings (`horizon-check.log`, `horizon-check.json`) |
| Embedded tests | `python3 tools/horizon.py test` | `test result: ok. 5 passed; 0 failed; 0 ignored; … 1240 filtered out` (`horizon-test.log`) |
| Fast build | `python3 tools/horizon.py build` | `cargo_exit: 0`, 7m20s (`horizon-build.json`) |
| E2E | `build-slot.sh e2e --job imfast --port ports/imfast -- bash tools/e2e.sh` | `PASS`, `E2E EXIT:0` (`e2e.log`) |
| E2E negative | same with `SELFTEST=1` | `FAIL`, `E2E EXIT:1` (`e2e-selftest.log`) |
| Log checker | `python3 ports/tools/check-e2e-logs.py imfast --strict` | `logs=2 ok=2 missing-exit=0 bad-exit=0`, exit 0 |
| Package | `python3 tools/package.py` | `imfast.hnmod`, 3,782 B, sha256 `bf94eba0b926eae25b782b84f3c0f97de1dcb4c60200b2be15b640e80374a347`, 5 entries; a rebuild gives the same bytes |

The retired tree (mixins, `.hnmod`, `compatibility.json`, `rust-toolchain.toml`, the old
`tools/`) is kept at `../_shared/evidence/imfast/retired-hnmod-0.1.0/` so the 0.1.0
evidence stays re-readable.

The e2e is a LAN host plus a real LAN guest (two headless games), because Horizon
exempts the single-player owner from the movement checks. It shows the same 200-block
guest move kept with the rules off and refused with them back on, and the unmodified game
starting with both rules on; the negative run inverts that assertion and exits 1.

No content half: the jar carries no assets or data to port (see `FINDINGS.md`).
