# HawkTUI v0.2.0 Implementation Plan

**Date**: 2026-04-03  
**Status**: Ready for Implementation  
**Investigation**: [investigation.md](./investigation.md)

---

## Executive Summary

The v0.2.0 implementation is **89% complete**. The main blocker is **test compilation errors** (14 errors) due to API changes in PiBridge. Once tests are fixed, we can commit and continue.

---

## Phase 1: Fix Test Compilation (PRIORITY)

**Estimated Time**: 15-20 minutes  
**Blocker**: Cannot run test suite until fixed

### Tasks

1. **Fix async method calls in tests** (10 errors)
   - File: `tests/integration_tests.rs`
   - Change `bridge.model()` → `bridge.model().await`
   - Change `bridge.provider()` → `bridge.provider().await`
   - Wrap in async test blocks (already async)

2. **Fix set_model() signature** (2 errors)
   - Change `bridge.set_model("model")` → `bridge.set_model("provider", "model").await`

3. **Remove set_provider() calls** (1 error)
   - Delete or replace with `set_model()` calls

4. **Add `partial` field to AssistantMessageEvent** (4 errors)
   - Add `partial: true` to test event constructors
   - Location: `src/providers/pi_bridge.rs` lines 602-625

---

## Phase 2: Complete Remaining TODO

**Estimated Time**: 5-10 minutes

### Task: Extract Session ID

File: `src/providers/pi_bridge.rs:256`

```rust
// Current:
"new_session".to_string() // TODO: Extract actual session ID

// Need to:
// 1. Check pi::sdk for session ID accessor
// 2. Extract from handle.session() or similar
```

**Investigation needed**: Check pi_agent_rust API for session ID retrieval.

---

## Phase 3: Clean Up & Commit

**Estimated Time**: 5 minutes

### Tasks

1. Remove unused import warning
   - File: `src/providers/pi_bridge.rs:25`
   - Remove `AbortSignal` from imports (unused)

2. Run `cargo fmt`

3. Run `cargo clippy` (should be clean)

4. Run `cargo test` (all tests pass)

5. Commit changes:
   ```bash
   git add -A
   git commit -m "v0.2.0: Real pi_agent_rust integration, fix test API compatibility"
   ```

---

## Phase 4: Integration Testing (Optional)

**Estimated Time**: 30-60 minutes

### Manual Testing

1. Build and run HawkTUI:
   ```bash
   cargo run --release
   ```

2. Test key features:
   - [ ] Streaming response displays correctly
   - [ ] Theme switching (`/theme hawk-light`)
   - [ ] Input history (Up/Down arrows)
   - [ ] Session management
   - [ ] Tool listing
   - [ ] Cancel streaming (Ctrl+C)

3. Verify pi_agent_rust connectivity:
   - [ ] Connects to default provider
   - [ ] Streams real responses
   - [ ] Tool execution works

---

## Phase 5: Update Documentation

**Estimated Time**: 10 minutes

### Tasks

1. Update `memory/architecture.json` with final state
2. Update `notes/code_review/feature_matrix.md` to 100% if applicable
3. Add release notes to CHANGELOG (if exists)

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| pi_agent_rust API changed | Medium | Check docs, may need updates |
| Session ID extraction unavailable | Low | Keep TODO, handle gracefully |
| Integration test requires live agent | Medium | Mock or skip integration tests |

---

## Success Criteria

- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes with no warnings
- [ ] All changes committed
- [ ] Manual run successful

---

## Next Steps After v0.2.0

1. **Session panel population** - Wire up to actual session list
2. **Tool panel population** - Wire up to actual tool registry
3. **Event processing** - Handle `AgentEvent` in event loop
4. **Context panel** - Implement file context management
5. **Branching** - Implement conversation branching
6. **Export** - Implement conversation export
7. **Cost tracking** - Implement token/cost tracking

---

## Estimated Total Time: 35-50 minutes
