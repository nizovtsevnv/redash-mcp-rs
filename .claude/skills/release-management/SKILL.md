# Release Management Skill

Release workflow for redash-mcp-rs: quality checks, version bump, changelog, tag, and GitHub release.

## When to Use

- Create a new release (patch/minor/major)
- Recreate a failed release tag
- Generate changelog from git history

## Prerequisites

- Clean working directory (or handle uncommitted changes explicitly)
- Rust toolchain available (`cargo fmt`, `cargo clippy`, `cargo test`, `cargo build`)
- `gh` CLI authenticated

## Workflow (8 Steps)

1. **Quality checks** — fmt, clippy, test, build
2. **Change analysis** — commits since last tag, uncommitted changes
3. **Version selection** — read current version, offer bump options
4. **Update Cargo.toml** — edit `version = "X.Y.Z"` on line 3
5. **Generate CHANGELOG.md** — Keep-a-Changelog format, show draft for approval
6. **Post-update quality checks** — re-run step 1
7. **Commit & tag** — `chore: release version X.Y.Z` + annotated tag
8. **Push & release** — confirm before push, `gh release create`

## Step-by-Step Implementation

### Step 1: Pre-Release Quality Checks

Run these commands in sequence. If ANY fail, stop and report errors:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

**Error handling:**
- `cargo fmt --check` fails: suggest running `cargo fmt`
- `cargo clippy` fails: show warnings, suggest fixing before release
- `cargo test` fails: show failed tests, abort release
- `cargo build --release` fails: show build errors, abort release

**Output to user:**
```
Pre-Release Quality Checks
  Code formatting (cargo fmt --check)
  Linter checks (cargo clippy)
  Test suite (N passed, M ignored)
  Release build

All quality checks passed. Proceeding with release...
```

### Step 2: Change Analysis

Analyze changes from two sources:

#### Commits since last tag
```bash
last_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

if [ -n "$last_tag" ]; then
    git log ${last_tag}..HEAD --oneline --no-decorate
else
    git log --oneline --no-decorate
fi
```

Categorize by conventional commit types:
- `feat:` → Added
- `fix:` → Fixed
- `refactor:`, `perf:` → Changed
- `docs:` → Changed (if user-facing)
- `chore:` → omit unless significant
- `BREAKING CHANGE:` → highlight

#### Uncommitted changes
```bash
git status --porcelain
git diff --stat
```

**Output structure:**
```
Changes Analysis:

Uncommitted Changes:
- Modified: src/main.rs

Commits Since X.Y.Z:
- abc1234 feat: add query execution tool
- def5678 fix: handle empty API responses

Categorized:
Added:
- Query execution tool
Fixed:
- Empty API response handling
```

### Step 3: Version Selection

Read current version from `Cargo.toml` line 3 and last git tag.

Show options using AskUserQuestion:

```
Current version: 0.1.0

Select release type:
1. patch (0.1.0 → 0.1.1) - Bug fixes, minor changes
2. minor (0.1.0 → 0.2.0) - New features, backwards compatible
3. major (0.1.0 → 1.0.0) - Breaking changes
4. custom - Enter specific version
5. recreate 0.1.0 - Recreate existing tag (for failed CI/CD)
```

**For custom**: prompt for version string, validate format X.Y.Z.

**For recreate**: ask confirmation, then:
```bash
git tag -d VERSION
git push origin :refs/tags/VERSION 2>/dev/null || true
```

### Step 4: Update Cargo.toml

Only one file needs a version edit. `Cargo.lock` will auto-update on next cargo command. `flake.nix` reads version from `Cargo.toml` via `builtins.fromTOML`.

Use the Edit tool on `Cargo.toml` line 3:
```
old: version = "OLD_VERSION"
new: version = "NEW_VERSION"
```

**Verification** — grep for old version to confirm replacement:
```bash
grep -r "OLD_VERSION" --exclude-dir=.git --exclude-dir=target --exclude=CHANGELOG.md .
```

If old version is found in unexpected places, warn the user.

### Step 5: Generate CHANGELOG.md

Generate a new section using Keep-a-Changelog format.

**If CHANGELOG.md exists**: read it, insert new section after the header line.

**If CHANGELOG.md does not exist**: create it with header.

**Section format:**
```markdown
## [NEW_VERSION] - YYYY-MM-DD

### Added
- [from feat: commits]

### Changed
- [from refactor:, perf: commits]

### Fixed
- [from fix: commits]

### Removed
- [deleted features/files if any]
```

Only include sections that have entries. Omit empty sections.

**Show draft to user** using AskUserQuestion:
```
Generated CHANGELOG entry:

## [0.2.0] - 2026-03-02

### Added
- Query execution tool
- Dashboard listing support

### Fixed
- Empty API response handling

Options:
1. Use as-is
2. Edit manually before proceeding
3. Regenerate
```

After approval, update the file. Also add/update the version link at the bottom:
```markdown
[NEW_VERSION]: https://github.com/nizovtsevnv/redash-mcp-rs/releases/tag/NEW_VERSION
```

### Step 6: Post-Update Quality Checks

Re-run all checks to ensure version bump and changelog didn't break anything:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

This also regenerates `Cargo.lock` with the new version.

If any check fails, abort and show errors.

### Step 7: Commit & Tag

**Stage specific files only:**
```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
```

**Create commit** using conventional format:
```bash
git commit -m "$(cat <<'EOF'
chore: release version NEW_VERSION

Major changes:
- [bullet points from changelog]
EOF
)"
```

**Verify commit:**
```bash
git log -1 --oneline
git show --stat HEAD
```

**Check tag doesn't already exist** (unless recreate mode):
```bash
git rev-parse NEW_VERSION >/dev/null 2>&1 && echo "Tag exists!" || echo "OK"
```

**Create annotated tag:**
```bash
git tag -a NEW_VERSION -m "Release NEW_VERSION"
```

**Verify tag:**
```bash
git show NEW_VERSION --no-patch
```

### Step 8: Push & Release

**Ask user before pushing** using AskUserQuestion:

```
Ready to Push Release

This will push to origin (github.com/nizovtsevnv/redash-mcp-rs):
- Commit: abc1234 chore: release version NEW_VERSION
- Tag: NEW_VERSION

Do you want to push now?
1. Yes, push and create GitHub release
2. No, keep local only
```

**If yes:**
```bash
git push && git push origin NEW_VERSION
```

Extract changelog section and create GitHub release:
```bash
gh release create NEW_VERSION \
  --repo nizovtsevnv/redash-mcp-rs \
  --title "Release NEW_VERSION" \
  --notes "$(awk '/^## \[NEW_VERSION\]/{flag=1; next} /^## \[/{flag=0} flag' CHANGELOG.md)"
```

**If no**, show manual commands:
```
Release prepared locally but not pushed.

To push later:
  git push && git push origin NEW_VERSION
  gh release create NEW_VERSION --title "Release NEW_VERSION" \
    --notes "$(awk '/^## \[NEW_VERSION\]/{flag=1; next} /^## \[/{flag=0} flag' CHANGELOG.md)"

To undo:
  git tag -d NEW_VERSION
  git reset --hard HEAD^
```

**Final report:**
```
Release NEW_VERSION Created

Commit: abc1234 chore: release version NEW_VERSION
Tag: NEW_VERSION

Links:
- Release: https://github.com/nizovtsevnv/redash-mcp-rs/releases/tag/NEW_VERSION
- Actions: https://github.com/nizovtsevnv/redash-mcp-rs/actions
```

## Error Handling

### Uncommitted Changes at Start

If `git status --porcelain` shows uncommitted changes, ask:
1. Include in release commit
2. Commit separately first (pause release)
3. Stash and continue
4. Cancel release

### Tag Already Exists (non-recreate mode)

Ask:
1. Cancel and use different version
2. Switch to recreate mode
3. Cancel release

### Network/Push Failures

Show error and provide manual commands to retry or undo.

## Key Design Decisions

- **Tag format**: `X.Y.Z` (no `v` prefix)
- **Single file update**: only `Cargo.toml` — `flake.nix` auto-syncs via `builtins.fromTOML`, `Cargo.lock` auto-updates
- **Explicit staging**: `git add Cargo.toml Cargo.lock CHANGELOG.md` — never `git add -A`
- **GitHub release**: `gh release create` with changelog body as description

## Tools Used

- `Read` — read files for version detection and changelog
- `Edit` — update version in Cargo.toml
- `Write` — create CHANGELOG.md if it doesn't exist
- `Bash` — run git commands, cargo commands, `gh` CLI
- `Grep` — find version occurrences for verification
- `AskUserQuestion` — interactive prompts for version selection, changelog approval, push confirmation
