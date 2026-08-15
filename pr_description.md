🎯 What:
Added a unit test `test_reset_context` to verify `Engine::reset_context()` correctly clears all engine state, including preedit buffers (`buffer`, `raw_buffer`), historic buffers (`last_committed_raw`, `last_committed_text`), the `uo_smart_fallback` value, and sets the state to `Idle`.

📊 Coverage:
Covers the exact behavior of `reset_context()` inside `vnikey-core/src/engine.rs`.

✨ Result:
Prevents future regressions around buffer leakages during window focus changes where `reset_context()` is heavily relied upon.
