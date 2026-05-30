# Endpoint Comparison: Original vs Current

Comparison of the original local-filesystem server (`docs/ORIGINAL_ENDPOINTS.md`) with the current GitHub-proxied server. The original uses `/api/` prefix; the current server mounts without it (the Netlify proxy handles the prefix).

---

## Endpoints Added (not in original)

### Authentication & Session

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/auth/start` | GitHub OAuth login initiation |
| GET | `/auth/callback` | GitHub OAuth callback |
| POST | `/auth/logout` | Clear session |
| GET | `/me` | Current user info |

### Webhooks

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/catalog-webhook` | GitHub webhook for catalog updates |
| POST | `/language-webhook` | GitHub webhook for language repo changes |

### Admin (PR Management)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/admin/pending-prs?language=` | List open PRs on language repo |
| GET | `/admin/pr-files?language=&pr=` | List files in a PR |
| POST | `/admin/approve-pr?language=&pr=` | Approve a PR |
| POST | `/admin/reject-pr?language=&pr=` | Reject a PR |

### User Language Management

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/user-languages/available-languages` | All catalog languages |
| GET | `/user-languages/my-languages` | User's claimed languages |
| POST | `/user-languages/claim-language/<code>` | Claim a language |
| POST | `/user-languages/release-language/<code>` | Release a language |
| GET | `/user-languages/current-language` | Active working language |
| POST | `/user-languages/current-language/<code>` | Switch working language (auto-claims if in catalog) |

### User Resource Selection

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/user-resources/my-resources` | User's selected resource paths |
| POST | `/user-resources/select-resource/<path>` | Add resource to selections |
| POST | `/user-resources/deselect-resource/<path>` | Remove resource from selections |

### Health

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Server health check |

---

## Endpoints Unchanged from Original

All of these categories are present and functionally identical:

- **Root & Clients** -- redirect, favicon, list-clients, client-interfaces, client-config, version
- **SSE notifications** -- `/notifications/`
- **Settings** -- languages, auth-token, typography, typography-feature
- **Net status** -- status, enable, disable
- **Debug** -- status, enable, disable
- **Navigation** -- BCV get/set
- **App State** -- current-project get/set/clear
- **Temp files** -- read/write
- **i18n** -- raw, negotiated, flat, untranslated, used-languages, post
- **Content Utils** -- templates, metadata-template, template-filenames, content-template, versifications, versification, product-content
- **LLM** -- model listing, RAG prompt
- **Video** -- obs-para, obs-story
- **Gitea** -- endpoints, remote-repos, user-remote-repos, login, logout, collaborators

---

## Endpoints with Changed Behavior

These endpoints exist in both versions but behave differently for curated org (`git.door43.org/*`) and GitHub-hosted (`github.com/*`) paths.

### Resource Listing

| Endpoint | Original | Current |
|----------|----------|---------|
| `GET /git/list-local-repos` | Filesystem walk only | Filesystem + curated orgs + GitHub; filtered to `selected_resources` |
| `GET /burrito/metadata/summaries` | Walks all local repos, optional `?org=` filter | Only fetches metadata for entries in `selected_resources`; returns `{}` if empty |

### Reading Content (Proxied)

| Endpoint | Original | Current |
|----------|----------|---------|
| `GET /burrito/metadata/summary/<path>` | Reads local `metadata.json` | Proxies from GitHub/Gitea for non-local paths |
| `GET /burrito/metadata/raw/<path>` | Reads local `metadata.json` | Proxies from GitHub/Gitea for non-local paths |
| `GET /burrito/ingredient/raw/<path>?ipath=` | Reads local file | Proxies from GitHub/Gitea; resolves user's working branch |
| `GET /burrito/ingredient/bytes/<path>?ipath=` | Reads local file | Proxies from GitHub/Gitea; resolves user's working branch |
| `GET /burrito/paths/<path>` | Walks local ingredients dir | Uses GitHub Tree API / Gitea Tree API for non-local paths |
| `GET /burrito/zipped/<path>` | Zips local repo | Returns 501 for GitHub paths; works for Gitea and local |

### Git Operations (No-op for Proxied Repos)

| Endpoint | Original | Current |
|----------|----------|---------|
| `POST /git/clone-repo/<path>` | Clones repo to local filesystem | No-op for curated/GitHub; records path in `selected_resources` |
| `POST /git/delete/<path>` | `remove_dir_all` on local path | Removes from `selected_resources` for curated/GitHub |
| `POST /git/pull-repo/<remote>/<path>` | Fetch + merge on local repo | Returns "up-to-date" for curated/GitHub |
| `GET /git/status/<path>` | Local git status | Returns `[]` (clean) for curated/GitHub |

### Resource Creation (GitHub-backed)

| Endpoint | Original | Current |
|----------|----------|---------|
| `POST /git/new-obs-resource` | Creates local repo with template files | Pushes to GitHub via App (bulk commit); auto-registers in `selected_resources`; auto-claims language |

### Writing Content

| Endpoint | Original | Current |
|----------|----------|---------|
| `POST /burrito/ingredient/raw/<path>` | Writes to local file | Uses `GithubEditFlow` for GitHub paths |
| `POST /burrito/ingredient/bytes/<path>` | Writes to local file | Uses `GithubEditFlow` for GitHub paths |
| `POST /burrito/ingredient/delete/<path>` | Deletes local file | Uses `GithubEditFlow` for GitHub paths |
| `POST /burrito/ingredients/delete/<path>` | Deletes local files | Uses `apply_bulk_op` for GitHub paths |
| `POST /burrito/ingredient/copy/<path>` | Copies local file | Uses `GithubEditFlow` for GitHub paths |
| `POST /burrito/ingredient/revert/<path>` | Reverts from local git | Uses `GithubEditFlow` for GitHub paths |
| `POST /burrito/metadata/remake-ingredients/<path>` | Regenerates local metadata | Uses `apply_bulk_op` for GitHub paths |
| `POST /burrito/ingredient/zipped/<path>` | Extracts zip to local | Uses `apply_bulk_op` for GitHub paths |
| `POST /burrito/zipped/<path>` | Imports zip to local | Uses `apply_bulk_op` for GitHub paths |

---

## Endpoints Still Local-Only (No GitHub/Curated Handling)

These endpoints would fail if called with a `github.com/*` or curated org path. They only work with repos on the local filesystem.

### Git Operations

| Endpoint | Description |
|----------|-------------|
| `GET /git/log/<path>` | Commit log |
| `GET /git/branches/<path>` | List branches |
| `GET /git/remotes/<path>` | List remotes |
| `POST /git/branch/<ref>/<path>` | Checkout branch |
| `POST /git/new-branch/<ref>/<path>` | Create + checkout branch |
| `POST /git/add-and-commit/<path>` | Stage + commit |
| `POST /git/copy/<path>` | Copy repo locally |
| `POST /git/remote/add/<path>` | Add remote (not relevant for GitHub mode) |
| `POST /git/remote/delete/<path>` | Remove remote (not relevant for GitHub mode) |
| `POST /git/push/<path>` | Push to remote (not relevant for GitHub mode) |

### Resource Creation (Local Only)

| Endpoint | Description |
|----------|-------------|
| `POST /git/new-text-translation` | Create text translation repo |
| `POST /git/new-bcv-resource` | Create BCV resource repo |
| `POST /git/new-scripture-book` | Add book to scripture translation |
| `POST /git/new-bcv-resource-book` | Add book to BCV resource |
| `POST /git/new-tcore-resource` | Create Translation Core resource |
| `POST /git/new-translation-plan-resource` | Create translation plan resource |

### Burrito Operations (Local Only)

| Endpoint | Description |
|----------|-------------|
| `GET /burrito/audit/<path>` | Validate burrito structure |
| `POST /burrito/remake_burrito_from_zip/<temp_id>/<path>` | Recreate burrito from zip |

---

## Branch Resolution for GitHub Reads

When reading from `github.com/*` paths, the server resolves which branch to read from:

1. If the user's GitHub login is stored (saved during OAuth), try `pankosmia-edit-{login}` first
2. If that branch exists, use it (shows in-progress work before PR merge)
3. Otherwise fall back to the repo's default branch

This means user-created content is visible immediately after creation, without waiting for PR approval.

---

## Key Architectural Differences

| Aspect | Original | Current |
|--------|----------|---------|
| Storage | Local filesystem | GitHub repos (via App) + Gitea (curated, read-only) |
| Auth | None required | GitHub OAuth session |
| Resource visibility | All local repos visible | Only `selected_resources` shown |
| Clone/download | Actually clones to disk | Records selection only (no local clone) |
| Writes | Direct filesystem | GitHub Contents API / Git Data API |
| Branch management | Local git operations | Server-managed `pankosmia-edit-{login}` branches |
