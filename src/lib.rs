//! Im-Fast for Horizon: the server's movement checks start off.
//!
//! Im-Fast (see `FINDINGS.md`) rewrites five constants of NeoForge's
//! `ServerGamePacketListenerImpl`, so a guest that moves faster than the server
//! expects is not sent back. Horizon reads two of those thresholds from the game
//! rules `player_movement_check` and `elytra_movement_check`, so this mod asks the
//! console to put both off, once for each open world.
//!
//! The rules are what [`runtime`] decides; this file is the glue that runs them at
//! the end of the open's first running server tick, which is where
//! `Commands::run_command` is allowed (no player's state is swapped in there,
//! unlike inside a player's hook). The other three constants (the two "moved
//! wrongly" checks and the vehicle's 100) have no rule yet; `notes.md` carries the
//! gap.
pub mod runtime;

use std::sync::Mutex;

use mod_api::{self, Hooks, ServerHost, ServerTickEvent, ServerTickPhase};

/// The mod has no settings: it holds the open world whose rules it already put off.
#[derive(Default)]
struct Mod {
    applied: Mutex<runtime::Applied>,
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// The authoritative server tick's phases: at the end of the first running
    /// tick of a world, turn the two movement-check rules off, then leave them
    /// alone, so an operator who sets one back keeps it for that session.
    fn server_tick(&self, host: &mut dyn ServerHost, event: &mut ServerTickEvent<'_>) {
        let world_running = match event.phase {
            ServerTickPhase::ServerTickEnd { world_running } => world_running,
            // A frozen step does not advance the world; `runtime` also refuses it.
            _ => return,
        };
        let lines = {
            // Only this expression runs under the lock: one pure call on the
            // port's own state, which cannot panic and so cannot poison it.
            let mut applied = self
                .applied
                .lock()
                .expect("the movement-check state lock is held for one pure call");
            applied.on_tick(event.context.epoch.get(), world_running)
        };
        let Some(lines) = lines else {
            return;
        };
        // A line the console answers `not set` for changes nothing: `gamerule`
        // fails when the rule already holds the wanted value, which is the end
        // state this wants, so the answer is not checked.
        for line in &lines {
            let _ = host.commands().run_command(line);
        }
    }
}
