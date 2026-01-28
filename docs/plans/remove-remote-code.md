# Plan: Remove Upstream "Remote" Code

Remove all remote/cloud sync functionality from this fork. We deploy as a single EC2 instance with local GitHub OAuth - the remote code is unused and adds confusion.

---

## Phase 1: Delete Directories & Workflows

**Delete:**
- `crates/remote/`
- `remote-frontend/`
- `.github/workflows/remote-deploy-dev.yml`
- `.github/workflows/remote-deploy-prod.yml`

**Success:** Directories and workflow files no longer exist.

---

## Phase 2: Remove Cargo Dependencies

**Update:**
- `Cargo.toml` - remove `"crates/remote"` from workspace members
- `crates/server/Cargo.toml` - remove `remote` dependency
- `crates/services/Cargo.toml` - remove `remote` dependency

**Success:** `cargo check --workspace` passes.

---

## Phase 3: Remove Services Layer

**Delete:**
- `crates/services/src/services/remote_client.rs`
- `crates/services/src/services/share/` (entire directory)
- `crates/services/src/services/share.rs`

**Update:**
- `crates/services/src/services/mod.rs` - remove module declarations
- `crates/services/src/lib.rs` - remove re-exports

**Success:** Services crate compiles.

---

## Phase 4: Update Deployment Layer

**Update `crates/deployment/src/lib.rs`:**
- Remove `SharePublisher` import
- Remove `RemoteClientNotConfigured` struct
- Remove `RemoteClientNotConfigured` from `DeploymentError`
- Remove `share_publisher()` from `Deployment` trait

**Update `crates/local-deployment/src/lib.rs`:**
- Remove `remote_client` and `share_publisher` fields
- Remove `VK_SHARED_API_BASE` initialization logic
- Remove `remote_client()` and `share_publisher()` methods

**Update `crates/local-deployment/src/container.rs`:**
- Remove `_publisher` parameter from `LocalContainerService::new()`

**Success:** Deployment crates compile.

---

## Phase 5: Update Server Routes

**Delete:**
- `crates/server/src/routes/organizations.rs`

**Update:**
- `crates/server/src/routes/mod.rs` - remove organizations module
- `crates/server/src/routes/oauth.rs` - remove handoff routes
- `crates/server/src/routes/projects.rs` - remove remote project routes
- `crates/server/src/routes/tasks.rs` - remove share_publisher usage

**Success:** Server compiles, local OAuth still works.

---

## Phase 6: Update Server Error Handling

**Update `crates/server/src/error.rs`:**
- Remove `RemoteClientError` variant from `ApiError`
- Remove `RemoteClientNotConfigured` conversion
- Remove all `RemoteClientError` match arms

**Success:** Error handling compiles.

---

## Phase 7: Update Type Generation

**Update:**
- `crates/server/src/bin/generate_types.rs` - remove remote type exports
- `crates/server/build.rs` - remove `VK_SHARED_API_BASE` env passing

**Success:** `pnpm run generate-types` succeeds.

---

## Phase 8: Frontend Cleanup

**Delete:**
- `frontend/src/lib/remoteApi.ts`
- `frontend/src/hooks/useProjectRemoteMembers.ts`
- `frontend/src/hooks/useOrganizationProjects.ts`
- `frontend/src/components/org/RemoteProjectItem.tsx`
- `frontend/src/components/dialogs/projects/LinkProjectDialog.tsx`
- `frontend/src/components/dialogs/tasks/ShareDialog.tsx`

**Update:**
- `frontend/src/lib/api.ts` - remove remote API methods
- `frontend/src/hooks/useProjectMutations.ts` - remove link mutations
- `frontend/src/hooks/useTaskMutations.ts` - remove share mutation
- UI components referencing deleted files

**Success:** `pnpm run check` and `pnpm run lint` pass.

---

## Phase 9: Configuration Cleanup

**Update `package.json`:**
- Remove `remote:*` scripts
- Remove `VITE_VK_SHARED_API_BASE` from dev scripts

**Update CI workflows:**
- `.github/workflows/test.yml` - remove remote checks
- `.github/workflows/pre-release.yml` - remove `VK_SHARED_API_BASE` secrets

**Success:** CI pipeline passes.

---

## Phase 10: Documentation

**Update:**
- `CLAUDE.md` - remove remote directory warnings
- `AGENTS.md` - remove remote references

**Success:** Docs accurately describe codebase.

---

## Final Verification

1. `cargo build --workspace`
2. `cargo test --workspace`
3. `pnpm run check`
4. `pnpm run lint`
5. `pnpm run generate-types`
6. `pnpm run dev` - app loads, OAuth works

---

## Notes

- Leave `projects.remote_project_id` and `tasks.shared_task_id` database columns (unused but harmless, avoids migration)
- Local GitHub OAuth (`/api/local-auth/*`) is separate from remote handoff and will continue working
