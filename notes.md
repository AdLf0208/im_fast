# Im-Fast — port notes (slice 1, migrated to the SDK format)

Ported from **I'm Fast 1.0.3** (`imfast-NEOFORGE-1.21.1-1.0.3.jar`, MIT,
`Gabriel-Hiss/Im-Fast` at `b79f2d3ae45dc6f1035b8690b8f9c8aef5b8c255`), Minecraft Java
1.21.1 NeoForge. The port targets Horizon 26.3 at the campaign pin. Clean-room Rust
rewrite from the mod's behaviour; what the mod does and its evidence are in
[`FINDINGS.md`](FINDINGS.md).

**Licence: MIT (`license = "MIT"` in the jar's `neoforge.mods.toml`, MIT text in the
repo's `LICENSE`, © 2024 Gabriel-Hiss) → rank 0, permissive.** Local-only until the
maintainer approves publishing (ARCHITECTURE.md §8); publishable under MIT with this
source. `LICENSE` (both copyright lines) and `LICENSE-NOTICE.md` (provenance) travel
**inside** the package and are carried into the `.hmod`.

**Format: the SDK package (2026-11-18).** This port was migrated from the retired
`.hnmod` container to the flat SDK package — see
[`## Migration to the SDK format`](#migration-to-the-sdk-format) for the verbatim
recipe and every trap. `src/runtime.rs` is **byte-identical** to the pre-migration
`mixins/imfast/runtime.rs` (sha256 `f86b58a9f3b0d7ac013b95a1ed451350e018ad0f78edcb5756e5dfbd1e7719b8`),
including the file inside the built package.

## Approach (from the pin's `docs/modpacks.md`)

- **Class `content`, tier `logic`**, core, P0 (`ports/_shared/source-map/atm10-sources.csv`
  row 226; `ports/ledger.csv`). It is pure server-side behaviour with no content of its
  own: the jar holds one mixin class, one mixin config and a stale template lang file —
  no blocks, items, recipes, tags or models, so **nothing is carried**.
- **Carry-over vs agent port:** the agent port is the whole job (there is nothing to
  carry). §"5. Code: one agent per mod" and the `logic` row of §"Tiers".
- **Why now:** rank 0, core, mechanics, a small faithful slice of a behaviour Horizon
  already half-supports (the movement-check rules).

## The slice, and why this one

Im-Fast's five `@ModifyConstant`s remove the server's movement validation. Horizon runs
the same checks, and two of the five thresholds (the player's 100 and the elytra 300) are
read from the game rules `player_movement_check` and `elytra_movement_check`
(`crates/horizon-client/src/game/lan.rs`: `Hub::movement_checks`). Slice 1 puts both rules
off once for each open world, from the server tick, through the console.

That is the smallest honest slice: it reproduces the mod's headline behaviour (a guest
that moves faster than the server expects is not sent back) with the engine's own switch,
and it is exactly where two of the five constants land. Nothing is faked, and no hook is
invented.

## Hooks

| Hook | Method | What it does |
|---|---|---|
| `server_tick` (server, listen) | `Hooks::server_tick(&self, host: &mut dyn ServerHost, event: &mut ServerTickEvent<'_>)` | At `ServerTickEnd` of the first **running** tick of a new world epoch, run the two `gamerule … false` lines through `host.commands().run_command(line)`. |

The rule decisions (which rules, the console line, once per epoch) are in
`src/runtime.rs` and unit-tested there; `src/lib.rs` is 20 lines of glue and no numbers.

## Deviations (deliberate)

1. **A world rule, not a constant.** Im-Fast is stateless: a mixin lowers a threshold for
   every session and leaves nothing behind. The port flips the rules the engine reads, so
   the choice is **saved with the world** (`game_rules.dat`) and survives the mod's
   removal; `/gamerule player_movement_check true` sets it back. The alternative — no port
   at all — would leave the behaviour missing, so this is the honest maximum at this pin.
2. **Applied once per open, then left alone.** An operator who turns a check back on keeps
   it for that session (a mixin could not allow that). `Applied::on_tick` is the state
   machine that makes it once per world epoch, unit-tested.
3. **No readout command.** Im-Fast has none; the e2e reads the rules through the vanilla
   `gamerule` query instead of inventing a mod command.
4. **Nothing carried.** The jar's only asset is a template lang file naming example
   blocks/items that do not exist, and it has no `@Mod` class; carrying it would ship junk.
   The package therefore has no `assets/` or `data/` directory at all — a native package
   with neither is valid.
5. **The state lives in the mod instance.** `Mod { applied: Mutex<Applied> }` rather than a
   `static`: the generated `hooks()` boxes one `Mod` for the session, and `Hooks` is
   `Send + Sync`.

## Gaps (why this is `ported-partial`, not `ported`)

Three of Im-Fast's five constants have **no lever at this pin**:

| Im-Fast constant | Horizon | Blocked by |
|---|---|---|
| player `0.0625` "moved wrongly" | `lan/hub.rs` `collide` (`:4597`, `:4636`) | no rule, no stamped hook |
| vehicle `100.0` "moved too quickly" | `game/lan_vehicles.rs:510-515` | no rule; `vehicle_moved` is stamped now but is a **listen** hook (`VehicleMovedEvent { player, kind, from, to }`), so it cannot permit a faster move |
| vehicle `0.0625` "moved wrongly" | `game/lan_vehicles.rs:529-541` | no rule, no stamped hook |

**Re-checked at `0b14f8a`, not assumed (2026-11-18).** The old note said
`native_player_move` was unstamped and fired after the validation. At this pin it **is**
stamped (`Hooks::player_move`, `contract = "cancel"`, `PlayerMoveEvent { player, from,
from_rotation, to, to_rotation, cancelled }`), and its own doc is explicit: it runs *after
the server's own validation and before the report is committed*, and setting `cancel`
**vetoes** the report (the client is corrected to the accepted position). So it can only
refuse more movement, never permit an over-fast move: implementing the rest of Im-Fast on
top of it would reproduce the bug, not the behaviour. What is missing is unchanged — a
**say on the thresholds themselves** for the player's "moved wrongly" check and both
vehicle checks.

**Issue shape:** *"Rules (or a pre-validation say) for the movement checks that carry no
game rule: the player's 'moved wrongly' 0.0625 and both vehicle thresholds;
`player_move` is stamped but is a post-validation veto, and `vehicle_moved` is listen-only."*

The second half of the mod is a re-slice at a later pin, not more work here.

## Migration to the SDK format

*Recipe written 2026-11-18 from this port, the first migration. Follow it verbatim for the
other 68 bundles; it is deliberately mechanical because only the glue differs.*

**Why.** Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992) retired `.hnmod`.
The registry closed every `.hnmod` submission on 2026-11-18 with one reason, and an
`.hnmod` no longer loads. The new unit is **one flat `.hmod`**: content, native code, or
both. Pins for this migration: nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`**
(2026-11-18), mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, launcher 0.2.78,
`cargo-horizon-mod` built from `horizon-rust/horizon@main` (`448cd693`).

### 0. Read the surface before writing glue

```sh
cargo horizon-mod new /tmp/opencode/<id>-new --template native   # fetches .horizon/sdk/
cargo horizon-mod hooks <text>            # search the catalog offline, e.g. `tick`, `run_command`
```

The declaration the generator reads is
`.horizon/sdk/api/crates/horizon-mod-vocab/src/hooks/<name>.rs`; the **generated** `Hooks`
trait, every event struct's fields and every dispatch function are in
`.horizon/sdk/api/crates/horizon-mod-vocab/generated/mod_api_generated.rs`
(`fn <name>(&self, host, event)` — read it, do not guess). Host methods and their access
levels are one file per area in `.horizon/sdk/api/crates/horizon-mod-vocab/src/host/`.

### 1. Files created

| Path | What it is |
|---|---|
| `horizon-mod.toml` | the only manifest and the only identity (see the field map below) |
| `Cargo.toml` | one crate; its only dependency is `horizon-mod-api` from `.horizon/sdk/` |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks`, and `pub mod runtime;` |
| `src/runtime.rs` | the port's pure logic, **copied byte-for-byte** (`cp`, then `cmp`) |
| `.gitignore` | `/target`, `/.horizon`, and `/Cargo.lock` (trap 5) |
| `LICENSE`, `LICENSE-NOTICE.md` | the licence travels inside the package (copy them in; the publish tree is not the package) |
| `icon.png` | optional, square, ≤512², ≤256 KiB — reuse the published one |

`assets/<id>/` and `data/<id>/` **only when the mod carries content**; a code-only port has
neither. `src/runtime.rs` is a module, so `lib.rs` must say `pub mod runtime;` — a private
module turns every `pub` item into a `dead_code` warning (trap 8).

### 2. Files deleted (and where the evidence went)

| Deleted | Why | Kept as evidence |
|---|---|---|
| `mixins/<id>/` (`horizon-mixin.toml`, `runtime.rs`, `support.rs`, one helper per hook) | the container is retired; `runtime.rs` moved to `src/` | `ports/_shared/evidence/<id>/retired-hnmod-0.1.0/` |
| `<id>.hnmod` | unloadable on launcher ≥ 0.2.70 | same folder (also in `_shared/horizon-mods/publish/<id>/`) |
| `compatibility.json` | `[native]` in the manifest is the pin now | same folder |
| `rust-toolchain.toml` | the pinned Horizon commit decides the toolchain | same folder |
| `tools/horizon.py`, `tools/package.py` | staging + `.hnmod` packaging are gone | same folder |
| `Cargo.lock` | not needed without registry deps; changes the artifact (trap 5) | not kept |

**Kept:** `tools/e2e.sh` (the scenario and its assertions are format-independent — point
`GAME=` at the launcher-built game), `notes.md`, `FINDINGS.md`, `VALIDATION.md`.

### 3. Manifest field mapping

| Old | New |
|---|---|
| `horizon-mixin.toml` `[mod] id` | `horizon-mod.toml` `id` (also the resource namespace; `[a-z0-9_]{1,64}`) |
| `[mod] version` | `version` (a **new** version for a resubmission: this port went `0.1.0` → `0.2.0`) |
| `[mod] priority = 100` | gone; ordering is `#[listener(priority = …)]` on a hook or `[dependencies]` `order` |
| `[compatibility] horizon_revision` | `[native] horizon` (exact 40 hex) |
| mixins revision from the bundle | `[native] mixins` (exact 40 hex) — **it moved too**: `5a64af93` → `e59da9094f0c…` |
| game version | `[native] game = "26.3"` |
| `[[module]] runtime/support` | **no plan section.** `src/runtime.rs` is an ordinary Rust module; no support module is needed when the hook body is short |
| `[[injection]] hook = …` + `helper = <file>.rs` + `bindings` | one `Hooks` method of the same name; `host` and `event` replace the bindings |
| `horizon-native.json` | gone (the manifest is the only one) |
| — | `api = "0.2"` (exact), `side`, `capabilities = ["registries"]` (fixed shared value), `license`, `homepage`, `issues` |

`side` is the **installation**, not the hook's logical side: an integrated server belongs
to the *client* installation, so a server-tick mod on a LAN host or in single player needs
`side = "both"` (trap 6). `capabilities = ["registries"]` is the one value the schema
allows even when a mod registers nothing (trap 7).

### 4. Hook → `Hooks` method

| Old injection | New method | Mapping notes |
|---|---|---|
| `[[injection]] hook = "server_tick", kind = "HEAD", helper = "server_tick.rs"`, bindings `game`, `context`, `phase` | `fn server_tick(&self, host: &mut dyn ServerHost, event: &mut ServerTickEvent<'_>)` | `event.phase` is `ServerTickPhase`; `event.context` is `&LifecycleContext` and `context.epoch` is a **`WorldEpoch`** (`epoch.get()`), not the old `Cell`; `context.tick` is a `TickId` |
| `game.native_run_command(&line)` | `host.commands().run_command(line)` | returns `bool`; `#[access(WorldOnly)]`, satisfied in `ModPhase::World`, where `server_tick` runs |
| `static APPLIED: Mutex<Applied>` in `support.rs` | a `Mutex<Applied>` field on the mod struct | the macro emits `Box::new(<Mod as Default>::default())`, so one instance lives for the session; `Hooks: Any + Send + Sync` ⇒ `Mutex`, not `RefCell` (trap 9) |

`Hooks` method signatures are **not** free: a hook that can be cancelled returns `Flow`, a
replacement returns `Option`, a collection adds to the event's list, a listen hook returns
`()`. Read the generated trait.

### 5. Build and check (the ladder for an SDK package)

`build-slot.sh` still speaks the old tool, so every command below runs **directly in the
port** with `CARGO_INCREMENTAL=0` — no staging, no staged tree, no `--target-dir`:

```sh
cd ports/<id>
# 1. compile + the pure tests, on the host target
CARGO_INCREMENTAL=0 cargo test                       # expect: runtime tests green, EXIT:0
# 2. lints and formatting, of THIS crate only
CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
cargo fmt --check                                    # NOT `--all` (trap 4)
# 3. package. `cargo horizon-mod` also fetches .horizon/sdk/ (gitignored) on first use.
rm -f Cargo.lock                                     # trap 5
CARGO_INCREMENTAL=0 cargo horizon-mod build          # -> target/horizon-mod/<id>.hmod
cp target/horizon-mod/<id>.hmod ./<id>.hmod          # the deliverable lives in the port
# 4. the registry's own checks on the delivered file
cargo horizon-mod check <id>.hmod                    # -> "<id> <version> (native-mod, format native-1/…)": EXIT:0
cargo horizon-mod check <id>.hmod --uses             # what it uses; the list is uses.json in the zip
cargo horizon-mod check <id>.hmod --json             # machine form
cargo horizon-mod fits <id>.hmod --build nightly     # "made for" that build, EXIT:0
sha256sum <id>.hmod
```

Measured for `imfast` (evidence in `ports/_shared/evidence/imfast/`):
`5 passed` unit tests; clippy 0 warnings; `fmt --check` clean; package
**7,570 B**, sha256 `f9bd961c261253f6c0f0502911b1f2bf91f1052f8332abab22fbce83d83f7510`,
8 entries (`horizon-mod.toml`, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`, `icon.png`,
`src/lib.rs`, `src/runtime.rs`, `uses.json`); `check` EXIT:0; `fits` EXIT:0.

`uses.json` is the best summary of a migration: this mod's is 24 entries — `hook:server_tick`
(+ `.context`, `.phase`), `method:Hooks::server_tick`, `method:Commands::run_command`,
`method:prelude::ServicesServerHost::commands`, the `LifecycleContext`/`ServerTickEvent`/
`ServerTickPhase`/`WorldEpoch`/`TickId`/`Authority` types, `handle:WorldEpoch::get`,
`handle:TickId::get`, `macro:horizon_mod`, `capability:registries`, four traits.

### 6. Traps, all hit in this migration

1. **A leftover old path refuses the package.**
   `error: unexpected package path: mixins/imfast/runtime.rs` — the project root may hold
   only package paths. Delete/move `mixins/` and the `.hnmod` *before* the first build.
2. **A 38-character hash reads as missing pins.**
   `error: horizon-mod.toml: native requires game and exact revisions` — the message does
   not say "too short". Copy both 40-hex commits from `sdk.json` / the scaffold; the mixins
   commit **is not** the old campaign pin (`e59da9094f0c…`, not `5a64af93…`).
3. **`cargo horizon-mod check` with no file checks wasm.**
   `error[E0463]: can't find crate for 'core' … the wasm32-unknown-unknown target may not
   be installed`. Use `check <file.hmod>` for the package and `cargo check`/`cargo test`
   for the native build.
4. **`cargo fmt --all --check` reports the SDK's files**, because `.horizon/sdk` is a path
   dependency: two `build.rs` diffs in `horizon-mod-api`/`horizon-mod-vocab`. Use
   `cargo fmt --check`; never "fix" the fetched SDK.
5. **`Cargo.lock` is packed and changes the sha256.** Any `cargo test`/`cargo clippy`
   writes one; `cargo horizon-mod build` then adds it to the zip and the artifact's hash
   changes from `f9bd961c…` to `6b81fdd0…`. Worse, the resolved versions are **newer** than
   the SDK's own `api/Cargo.lock` (`libc` 0.2.191 vs 0.2.190, `memchr` 0.4.35 vs 0.4.34,
   `siphasher` 1.0.154 vs 1.0.151, `zerocopy` 0.8.63 vs 0.8.60 …), i.e. a lock the SDK was
   not built against. A mod with no registry dependency of its own must be packaged
   **without** one: `rm -f Cargo.lock` right before the build, and `/Cargo.lock` in
   `.gitignore`.
6. **`side` is the installation side.** `side = "server"` would keep the mod off every
   client installation, so single player and a LAN host would lose it (an integrated
   server belongs to the client installation). Use `"both"` for anything but a
   dedicated-only mod.
7. **`capabilities = ["registries"]` is not optional** even for a mod that registers
   nothing: the shared manifest has exactly that value and unknown values are refused. It
   is a schema requirement, not a claim about content — say so in the manifest comment so
   the next reader does not "correct" it.
8. **`runtime.rs` as a module.** As the old crate's lib root its `pub` items were the
   public API; inside a private `mod runtime;` every one of them warns `dead_code`. Declare
   `pub mod runtime;` (the file itself stays byte-identical — do not add an
   `#[allow(dead_code)]` to it).
9. **`Hooks: std::any::Any + Send + Sync`.** Session state in the mod struct must be
   `Mutex`, not `RefCell`/`Cell`; the macro's `hooks()` boxes one `Default` instance, so the
   state survives across calls and dies with the session.
10. **`ServerTickPhase` is `#[non_exhaustive]`.** A `match` needs a wildcard
    (`_ => return`), and `ServerTickStart` must not be treated as "end of tick". The old
    `let … else` shape still compiles, but the wildcard match is the readable form.
11. **The context types changed shape.** `LifecycleContext { authority, epoch: WorldEpoch,
    tick: TickId }` with **public `get()`** on the newtypes; the old
    `NativeLifecycleContext.epoch` was a `Cell<u64>` (`.get()` worked there by accident of
    naming). Nothing else from `game::native_lifecycle` exists.
12. **`commands()` is `#[access(WorldOnly)]`.** `access_allowed` returns `true` for *every*
    operation in `ModPhase::World`, so a server-tick hook may call it; the same call from
    `register` (registration phase) is refused. Do not move the call into `register`.
13. **No `rust-toolchain.toml`.** The package deliberately has none: the pinned Horizon
    commit (and the launcher) decide the toolchain. `sdk.json` declares rust 1.99.1; this
    crate was compiled here with rustc 1.97.0 and is edition 2024.
14. **`cargo horizon-mod build` does pack a native project**, contrary to the *pinned*
    `docs/mods.md` text ("it refuses a folder with a `Cargo.toml`") — that text predates
    the merge. The new tool packs `horizon-mod.toml`, `Cargo.toml`, `src/`, the generated
    `uses.json`, and root files such as `LICENSE`/`icon.png`; it does not compile anything.
15. **The e2e could not be re-run here** — it needs a *launcher-built* game
    (`horizon-launcher native-build <port> --instance <i> --trust-native-code`), and
    `horizon-launcher` is not installed in this environment. `tools/e2e.sh` is kept
    unchanged (its assertions are the behaviour's proof) and must be run with
    `GAME=<launcher-built horizon-client>`; record that run before claiming `ported`.

## Verification

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

| Check | Command | Result |
|---|---|---|
| Runtime unit tests (SDK crate) | `cargo test` | **5 passed**, 0 failed (`evidence/imfast/sdk-test.log`) |
| Lints | `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings (`evidence/imfast/sdk-clippy.log`) |
| Formatting | `cargo fmt --check` | clean (`evidence/imfast/sdk-fmt.log`) |
| Package | `cargo horizon-mod build` | `target/horizon-mod/imfast.hmod`, EXIT 0 (`evidence/imfast/sdk-build.log`) |
| Registry check | `cargo horizon-mod check imfast.hmod` | `imfast 0.2.0 (native-mod, format native-1/0b14f8a/e59da90, for 26.3)`, **7,570 B**, sha256 `f9bd961c…7510`, **EXIT 0** (`evidence/imfast/sdk-check.log`, `sdk-check-json.log`) |
| What it uses | `cargo horizon-mod check imfast.hmod --uses` | 24 surface entries (`evidence/imfast/sdk-uses.json`), EXIT 0 |
| Fits the pinned build | `cargo horizon-mod fits imfast.hmod --build nightly` | `0b14f8a made for`, EXIT 0 (`evidence/imfast/sdk-fits.log`) |
| `runtime.rs` untouched | `unzip -p imfast.hmod src/runtime.rs \| sha256sum` | `f86b58a9…19b8` = the retired file, byte for byte |
| Fresh-clone repeat | `.horizon/`, `target/` and `Cargo.lock` removed, then `cargo horizon-mod build` | SDK fetched by itself; same 7,570 B, same sha256 (`evidence/imfast/sdk-fresh-clone.md`) |
| Symbols line | `python3 ports/tools/check-skills.py --notes ports/imfast` | `Skills line present`, **SKILLS GATE: PASS** |

### Not re-run in this migration

- **The in-game e2e** (host + real LAN guest, the 200-block jump differential, the
  negative run) — no `horizon-launcher` here to build a game carrying the mod, see trap 15.
  The 0.1.0 run stands as the behaviour's evidence (`evidence/imfast/e2e.log`, `e2e-selftest.log`,
  `python3 ports/tools/check-e2e-logs.py imfast --strict` → `logs=2 ok=2`): what changed is
  the *transport* of the same two console lines (a typed host method instead of a
  `Game` call), not the rules, which are byte-identical and tested.
- **The staged ladder** (`tools/horizon.py stage/check/test/build`) — those tools are
  deleted with the retired format.

### What the 0.1.0 e2e shows (a LAN host and a real LAN guest, `--headless`)

The movement checks only run for a guest (`shouldCheckPlayerMovement` exempts the
single-player owner), so the run is a host and a guest, and the "too quickly" limit is 100
blocks; the guest writes its position 200 blocks along x.

```
== the mod puts both movement checks off, at world start ==
player_movement_check=false elytra_movement_check=false
== a guest that jumps 200 blocks with the checks off is not sent back ==
guest x before: -60.5
guest x after: 139.5 (target 139.5)
== the same jump with the checks on is refused, so the rule is the switch ==
guest x before: 139.5
guest x after: 139.5 (the server sent it back)
== leg B: the unmodified game starts with both checks on ==
player_movement_check=true elytra_movement_check=true (no mod)
PASS: …
E2E EXIT:0
```

So the differential is real in one run: the same jump, in the same world, is kept with
the rules off (the mod's setting) and refused once the host sets them back on, and the
unmodified game starts with both on. The **negative** run inverts the first assertion (the
guest must have been sent back) and fails with `EXIT:1`, so the assertion is not vacuous.

## Files

- `horizon-mod.toml` — the only manifest: `id = "imfast"`, `version = "0.2.0"`,
  `api = "0.2"`, `side = "both"`, and `[native]` pinned to `0b14f8a` / `e59da90`.
- `Cargo.toml` — one crate; the only dependency is `horizon-mod-api` from `.horizon/sdk/`.
- `src/lib.rs` — the one `#[mod_api::horizon_mod] impl Hooks`: `server_tick`.
- `src/runtime.rs` — the pure decision (rules, console line, once per epoch), byte-identical
  to the pre-migration file, with its 5 tests.
- `LICENSE`, `LICENSE-NOTICE.md`, `icon.png` — inside the package.
- `tools/e2e.sh` — the in-game scenario, unchanged.
- `imfast.hmod` — the delivered package, 7,570 B, sha256
  `f9bd961c261253f6c0f0502911b1f2bf91f1052f8332abab22fbce83d83f7510`.
- `FINDINGS.md`, `VALIDATION.md`, this file.
- Evidence: `../_shared/evidence/imfast/` (`sdk-*.log`, `sdk-uses.json`,
  `sdk-sha256.txt`, the retired 0.1.0 tree under `retired-hnmod-0.1.0/`). Jar kept at
  `../_shared/licence-cache/imfast-NEOFORGE-1.21.1-1.0.3.jar`
  (sha256 `59bef045…`, sha1 `f8e92ef6…`).
