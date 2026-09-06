# /jobo/repos/thaum-renderer/domain/debug-log

## purpose
process-global leveled, categorized debug logging for every host of the renderer: five levels, console sink, optional file sink, capped in-memory buffer. Call sites are one line; nothing threads a logger through signatures.

## owns
- the `DebugLogLevel` / `DebugLogConfig` surface and level filtering
- the one process-global config + capped buffer (behind `RwLock`)
- console and file sink emission
- the `THAUM_DEBUG` env-var toggle (value = minimum level name; unset or `off` = disabled)

## does not own
- log file locations — hosts pick and configure `file_path` (the painter points it into its per-run `context/debug-logging/<stamp>/` folder)
- log rotation/culling — host-owned
- panic hooks — host-owned (the painter's hook reports through this seam)

## children-encapsulations
- none

## contents
- `debug_log.rs`
  - the whole seam (config, buffer, sinks, level helpers, tests)
- `contract.md`
  - this file

## dependencies
- std only

## exposed interfaces
- `log(level, system, message)` plus `error/warn/info/debug/trace` one-liners
- `configure_from_env()` — reads `THAUM_DEBUG`; console-only, idempotent boot helper for standalone apps
- `set_config(config)` / `config()` / `recent_lines()`

## interface consumers
- painter `orchestration/entrypoint/src/run_log.rs` (host wiring)
- painter `domain/lib.rs` re-exports this module as `thaum_painter_domain::debug_log`

## notes
- Moved down from the painter's `domain/debug-log/` so the renderer can log into the same sink as its host and standalone renderer apps get logging for free. Env var generalized `THAUM_PAINTER_DEBUG` → `THAUM_DEBUG` in the same move (no other references existed).
- Warn/Error reach the console even when disabled so operator-visible failures never depend on the toggle.
