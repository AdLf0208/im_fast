# Im-Fast — what the mod does (established before any port code)

Written from the **jar** (provenance below) and the **upstream repository**, not from
guesses. Nothing here is copied into the Rust; it is the behaviour the port must reproduce.

## Provenance

| | |
|---|---|
| Jar | `imfast-NEOFORGE-1.21.1-1.0.3.jar` (kept at `../_shared/licence-cache/`) |
| SHA-256 | `59bef0451a2045cc357b601fb1ff88da60e761867b1fd6eeaff63fa215b51c8b` |
| SHA-1 | `f8e92ef61de3751f619238eed5ae9952b5a049d0` |
| Mod id / version | `imfast` / `1.0.3` (`META-INF/neoforge.mods.toml`) |
| Licence in the jar | `license = "MIT"` — **rank 0**; the repo's `LICENSE` is the MIT text, © 2024 Gabriel-Hiss |
| Repo | <https://github.com/Gabriel-Hiss/Im-Fast>, commit `b79f2d3ae45dc6f1035b8690b8f9c8aef5b8c255` (2026-08-27, "fix(crash): fix modid error") |
| Jar contents | `META-INF/neoforge.mods.toml`, `imfast.mixins.json`, `com/biel/mod/mixin/VelocityMixin.class`, `assets/imfast/lang/en_us.json` — **no** blocks, items, recipes, tags, data or models |

The jar is exactly the repo built at that commit: one mixin class, one mixin config, a
stale template lang file (`itemGroup.imfast` / `example_block` / `example_item`, no
`@Mod` class in the repo either — the entry point is the mixin config alone).

## What it changes

`mods.toml`: *"A modern solution using mixin to remove minecraft speed limit"*. The whole
mod is `VelocityMixin`, five `@ModifyConstant`s (`priority = 2000`):

| Method | Constant | Becomes | Vanilla meaning |
|---|---|---|---|
| `ServerGamePacketListenerImpl.handleMovePlayer` | `float 100.0F` | `Float.MAX_VALUE` | "moved too quickly" limit, walking/flying |
| `ServerGamePacketListenerImpl.handleMovePlayer` | `float 300.0F` | `Float.MAX_VALUE` | "moved too quickly" limit, elytra |
| `ServerGamePacketListenerImpl.handleMoveVehicle` | `double 100.0` | `Double.MAX_VALUE` | vehicle "moved too quickly" |
| `ServerGamePacketListenerImpl.handleMovePlayer` | `double 0.0625` | `Double.MAX_VALUE` | "moved wrongly" (collision resolved the move somewhere else) |
| `ServerGamePacketListenerImpl.handleMoveVehicle` | `double 0.0625`, `ordinal = 1` | `Double.MAX_VALUE` | vehicle "moved wrongly" |

So the behaviour is **server-side movement-validation relaxation**: a guest's position
packet no longer trips `moved too quickly!`/`moved wrongly!`, so the server does not send
the player back. It is a stateless threshold rewrite — no config file, no command, no
persistent state, no client half (the mixin is in the common set), and nothing to render.

**Where it is felt.** `shouldCheckPlayerMovement` exempts the single-player owner, so the
mod does nothing in a plain single-player world; its effect is on **LAN guests and
dedicated servers** (and, seen from the other side, on a client that a modded server no
longer pulls back).

## Where Horizon does the same work

| Check | Horizon | Gated by |
|---|---|---|
| player "moved too quickly" (100 / 300 elytra) | `crates/horizon-client/src/lan/hub.rs:4321-4348` (`position_change`) | **game rules** `player_movement_check` and `elytra_movement_check` (`game/lan.rs:2386`, `hub.rs:4312`) |
| player "moved wrongly" (0.0625) | `lan/hub.rs:4349` + `collide` (`hub.rs:4597`, `:4636`) | the player not being creative — **no rule, no hook** |
| vehicle "moved too quickly" (100) | `crates/horizon-client/src/game/lan_vehicles.rs:510-515` | only the single-player owner — **no rule, no hook** |
| vehicle "moved wrongly" (0.0625) | `game/lan_vehicles.rs:529-541` | as above — **no rule, no hook** |

## What the pin lets a mod reach, and what it does not

Stamped hooks (`native-hooks.json`, 432) that touch a guest's move:

- `player_velocity` (v1, listen) — the server syncing a player's velocity *after* it is
  decided. Cannot change a threshold.
- `vehicle_moved` (v1, listen) — heard around the vehicle move, listen-only.
- `net_from_client`/`net_from_server` — mod message channels, not play packets.
- `world_game_rule_change` (v1, listen) — hears a rule change, cannot make one.

Pending (in the source, **not stamped** at this pin, so a bundle may not name them):
`native_player_move` (Bukkit's `PlayerMoveEvent`) — and its own doc says it is *"heard
after ... passed the server's checks (moved too quickly, moved wrongly, the anticheat)"*,
so even once stamped it could not un-do a pull-back; and `vehicle_moved_v2`, which does
have a say on the move.

**The levers that exist today** for the two rules above: `/gamerule player_movement_check
<false|true>` and `/gamerule elytra_movement_check <false|true>`, reachable from a server
hook through `native_run_command` (API 21, console permission 4, refused only inside a
player's hook). Two of Im-Fast's five constants — the 100 and the 300 — are exactly the
two rules Horizon reads. The other three (both "moved wrongly", the vehicle 100) have no
lever at this pin.

## Slice 1 (the smallest honest slice) and why

Ship a server-side bundle that turns `player_movement_check` and `elytra_movement_check`
off once per open world, at the first running server tick, and says so in `notes.md`.
That reproduces the mod's headline behaviour (a fast player is not sent back) with the
engine's own switch, and it is where the two most-felt constants land. It is
**`ported-partial`**, not `ported`:

- reproduced: player "moved too quickly" (100 and elytra 300), so no pull-back for a fast
  or lagging player;
- not reproduced: the two "moved wrongly" thresholds and the vehicle "moved too quickly"
  (game/lan_vehicles.rs) — no stamped hook, no rule; the gap request is
  *"a say on a guest's move before the server validates it (or rules for 'moved
  wrongly' and the vehicle checks), since `native_player_move` is heard after the
  checks"*.

Deliberate deviations (recorded in `notes.md`): the port flips a **world rule** rather
than a constant, so the choice is saved with the world and an operator can turn the
checks back on for a session (the mod does not fight them); Im-Fast is stateless.
