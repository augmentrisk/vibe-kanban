# Plan: Remove Upstream "Remote" Code from Vibe Kanban Fork

## Background

Vibe Kanban's upstream codebase includes a "remote" subsystem designed for their cloud SaaS offering. This enables:
- Cross-workspace task sharing between multiple local VK instances
- Organization and team management via a central cloud API
- OAuth handoff for cloud authentication
- Real-time sync via Electric SQL

**Our fork doesn't use any of this.** We deploy as a single EC2 instance with local GitHub OAuth authentication for multiple users. The remote code adds confusion, maintenance burden, and dead code paths.

## Goal

Surgically remove all remote-related code while preserving the local multi-user deployment model. The application should function identically after removal - users won't notice any difference since these features were never enabled.

---

## Phase 1: Remove Upstream Directories

**What:** Delete the two main upstream-only directories that contain the remote service and its frontend.

**Scope:**
- `crates/remote/` - The entire remote API service crate
- `remote-frontend/` - The remote service's React frontend

**Success Criteria:**
- Directories no longer exist in the repository
- No broken symlinks or references remain
- Git history preserved (normal deletion, not history rewrite)

---

## Phase 2: Remove CI/CD Workflows

**What:** Delete GitHub Actions workflows that deploy the remote service.

**Scope:**
- `.github/workflows/remote-deploy-dev.yml`
- `.github/workflows/remote-deploy-prod.yml`

**Success Criteria:**
- No workflows referencing remote deployment exist
- Remaining workflows continue to pass
- No orphaned secrets references in workflow files

---

## Phase 3: Remove Rust Dependencies

**What:** Remove the `remote` crate from the Cargo workspace and dependent crates.

**Scope:**
- Root `Cargo.toml` workspace members
- `crates/server/Cargo.toml` dependencies
- `crates/services/Cargo.toml` dependencies

**Success Criteria:**
- `cargo build --workspace` succeeds without the remote crate
- No compilation errors referencing missing `remote::` imports
- Cargo.lock updated to exclude remote crate dependencies

---

## Phase 4: Remove Services Layer Integration

**What:** Remove the RemoteClient HTTP client and SharePublisher task-sharing service from the services crate.

**Scope:**
- `crates/services/src/services/remote_client.rs` - HTTP client for remote API
- `crates/services/src/services/share/` - Task sharing publisher module
- Module exports in `mod.rs` and `lib.rs`

**Success Criteria:**
- Services crate compiles without remote-related modules
- No exports of `RemoteClient`, `RemoteClientError`, `SharePublisher`, `HandoffErrorCode`
- Dependent crates updated to not import these types

---

## Phase 5: Update Deployment Abstraction

**What:** Remove remote-related methods from the Deployment trait and LocalDeployment implementation.

**Scope:**
- `Deployment` trait: remove `share_publisher()` method
- `LocalDeployment`: remove `remote_client` and `share_publisher` fields
- `LocalDeployment`: remove `remote_client()` and `share_publisher()` methods
- `LocalDeployment`: remove `VK_SHARED_API_BASE` initialization logic
- `LocalContainerService`: remove unused `_publisher` parameter
- `DeploymentError`: remove `RemoteClientNotConfigured` variant
- `RemoteClientNotConfigured` struct: delete entirely

**Success Criteria:**
- `crates/deployment` compiles without SharePublisher import
- `crates/local-deployment` compiles without remote client initialization
- No runtime checks for `VK_SHARED_API_BASE` environment variable

---

## Phase 6: Remove Server Routes

**What:** Remove API routes that proxy to the remote service or use remote functionality.

**Scope:**
- DELETE `crates/server/src/routes/organizations.rs` entirely (all endpoints use remote client)
- `oauth.rs`: remove `handoff_init` and `handoff_complete` routes
- `projects.rs`: remove remote project linking routes
- `tasks.rs`: remove shared task update logic
- `mod.rs`: remove organizations router merge

**Success Criteria:**
- All removed endpoints return 404 (not 500)
- Local GitHub OAuth continues to work (`/api/local-auth/*` routes)
- Local project and task CRUD operations work normally
- No references to `deployment.remote_client()` or `deployment.share_publisher()` remain

---

## Phase 7: Update Server Error Handling

**What:** Remove error types and handlers for remote client errors.

**Scope:**
- `ApiError` enum: remove `RemoteClient` variant
- Remove `impl From<RemoteClientNotConfigured> for ApiError`
- Remove `RemoteClientError` match arms in status code and message formatting

**Success Criteria:**
- Error handling compiles without remote error types
- All remaining error paths have appropriate HTTP status codes
- No dead code warnings for unused error variants

---

## Phase 8: Update Type Generation

**What:** Remove remote types from the TypeScript type generation and build scripts.

**Scope:**
- `generate_types.rs`: remove remote type declarations
- `build.rs`: remove `VK_SHARED_API_BASE` environment variable passing
- `shared/types.ts`: regenerate without remote types

**Success Criteria:**
- `pnpm run generate-types` succeeds
- `shared/types.ts` contains no remote-related types
- Frontend type imports resolve correctly

---

## Phase 9: Remove Utils Types (If Unused)

**What:** Audit and remove remote-related types from the utils crate if they're no longer used.

**Scope:**
- `crates/utils/src/api/organizations.rs` - Organization types
- `crates/utils/src/api/projects.rs` - RemoteProject types
- Related type exports

**Success Criteria:**
- No unused type warnings
- Only types actually used by remaining code are kept
- Consider keeping types if database columns reference them (avoid migration)

---

## Phase 10: Frontend Cleanup

**What:** Remove frontend components, hooks, and API methods related to remote functionality.

**Scope:**

*Files to delete:*
- `frontend/src/lib/remoteApi.ts`
- `frontend/src/hooks/useProjectRemoteMembers.ts`
- `frontend/src/hooks/useOrganizationProjects.ts`
- `frontend/src/hooks/useUserOrganizations.ts` (if remote-only)
- `frontend/src/components/org/RemoteProjectItem.tsx`
- `frontend/src/components/dialogs/projects/LinkProjectDialog.tsx`
- `frontend/src/components/dialogs/tasks/ShareDialog.tsx`

*Files to update:*
- `frontend/src/lib/api.ts` - remove remote API methods and type imports
- `frontend/src/hooks/useProjectMutations.ts` - remove link mutations
- `frontend/src/hooks/useTaskMutations.ts` - remove share mutation
- `frontend/src/hooks/useAssigneeUserName.ts` - remove remote assignee logic
- `frontend/src/components/projects/ProjectCard.tsx` - remove "Link" button
- `frontend/src/pages/settings/OrganizationSettings.tsx` - update or simplify

**Success Criteria:**
- `pnpm run check` passes (TypeScript compilation)
- `pnpm run lint` passes (no unused imports/variables)
- No UI elements reference sharing or remote linking
- Application loads without console errors

---

## Phase 11: Update Localization

**What:** Remove translation keys for removed features across all locales.

**Scope:**
- All files in `frontend/src/i18n/locales/*/projects.json`
- Remove `linkDialog`, `shareDialog`, and related translation keys

**Success Criteria:**
- No missing translation warnings at runtime
- No orphaned translation keys
- i18n files are consistent across all locales (en, es, fr, ja, ko, zh-Hans, zh-Hant)

---

## Phase 12: Configuration and Build Cleanup

**What:** Remove remote-related npm scripts, environment variables, and CI configuration.

**Scope:**

*package.json:*
- Remove `remote:*` npm scripts
- Remove `VITE_VK_SHARED_API_BASE` from dev scripts

*CI Workflows:*
- `.github/workflows/test.yml` - remove remote type/db checks
- `.github/workflows/pre-release.yml` - remove `VK_SHARED_API_BASE` secret usage

**Success Criteria:**
- `pnpm run dev` works without remote environment variables
- CI pipeline passes without remote-related steps
- No references to `VK_SHARED_API_BASE` or `VITE_VK_SHARED_API_BASE` in codebase

---

## Phase 13: Documentation Updates

**What:** Update documentation to reflect the removal of remote functionality.

**Scope:**
- `CLAUDE.md` - Remove warnings about remote directories (they'll be gone)
- `AGENTS.md` - Update to reflect simplified architecture
- `README.md` (if exists) - Remove any remote setup instructions
- `docs/` - Audit for remote references

**Success Criteria:**
- Documentation accurately describes the current codebase
- No instructions for features that no longer exist
- Clear explanation that this fork is local-deployment only

---

## Phase 14: Test Suite Audit

**What:** Ensure all tests pass and remove any remote-specific tests.

**Scope:**
- Rust tests: `cargo test --workspace`
- Frontend tests: `pnpm run test` (if configured)
- Remove any test files specific to remote functionality
- Update test fixtures that reference remote types

**Success Criteria:**
- `cargo test --workspace` passes with no failures
- No tests reference removed modules
- Test coverage for remaining functionality is maintained

---

## Phase 15: Final Verification

**What:** End-to-end verification that the application works correctly.

**Verification Checklist:**
1. `cargo build --workspace` - Rust compilation succeeds
2. `cargo check --workspace` - Type checking passes
3. `cargo test --workspace` - All tests pass
4. `pnpm run check` - Frontend TypeScript compilation succeeds
5. `pnpm run lint` - No linting errors
6. `pnpm run generate-types` - Type generation works
7. `pnpm run dev` - Application starts without errors
8. Manual test: GitHub OAuth login works
9. Manual test: Create/read/update/delete projects
10. Manual test: Create/read/update/delete tasks
11. Manual test: No console errors in browser
12. Manual test: No "remote" or "share" UI elements visible

**Success Criteria:**
- All automated checks pass
- All manual tests pass
- Application is functionally identical to before (minus unused features)

---

## Database Considerations

The following database columns are remote-related but can remain unused:
- `projects.remote_project_id`
- `tasks.shared_task_id`

**Recommendation:** Leave these columns in place. Removing them requires a database migration, and unused nullable columns cause no harm. This can be addressed in a future cleanup if desired.

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking local OAuth | Low | High | Remote handoff is separate from local GitHub OAuth |
| Type generation failure | Medium | Medium | Regenerate types after each phase |
| Missing UI removal | Medium | Low | Thorough grep for "remote", "share", "link" |
| Broken tests | Medium | Medium | Run test suite after each major phase |
| Database issues | Low | High | Don't modify schema; leave unused columns |

---

## Rollback Plan

If issues arise:
1. Git revert to the commit before remote removal began
2. The remote code was dormant, so reverting restores a working state
3. No data migration is involved, so no data loss risk

---

## Estimated Effort

- **Phase 1-2:** Low - simple deletions
- **Phase 3-8:** Medium - Rust refactoring requiring careful dependency management
- **Phase 9-11:** Medium - Frontend cleanup with many files
- **Phase 12-15:** Low - configuration and verification

**Total:** Approximately 30-40 files affected across backend and frontend.
