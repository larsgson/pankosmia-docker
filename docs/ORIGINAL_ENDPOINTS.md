# Pankosmia Web API Endpoints

All endpoints are served by a Rocket web server (default port 19119). Responses are JSON unless noted otherwise.

Legend for **Filesystem** column:
- **Read** - reads from local filesystem
- **Write** - writes/modifies local filesystem
- **Delete** - removes files from local filesystem
- **None** - no local filesystem interaction
- **Remote** - calls a remote API

---

## Root & Client Serving

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| GET | `/` | Redirects to client main interface | None |
| GET | `/api/favicon.ico` | Serves favicon | Read |
| GET | `/api/list-clients` | Lists registered clients | Read |
| GET | `/api/client-interfaces` | Returns public URL interfaces offered by clients | Read |
| GET | `/api/client-config` | Returns client configuration settings | Read |
| GET | `/api/version` | Returns package, product, and resource version info | Read |

Static file mounts:

| Mount Point | Description | Filesystem |
|-------------|-------------|------------|
| `/api/webfonts/` | Serves custom web fonts | Read |
| `/api/app-resources/` | Serves application resources | Read |
| `/clients/<client_id>/` | Serves compiled client applications | Read |

---

## SSE Notifications

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| GET | `/api/notifications/` | Opens Server-Sent Events stream for real-time notifications | None |

---

## Settings (`/api/settings`)

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| GET | `/api/settings/languages` | Returns current UI language settings | None |
| POST | `/api/settings/languages/<languages..>` | Sets UI languages | **Write** |
| GET | `/api/settings/auth-token/<token_key>/<code>/<client_code>` | OAuth landing page for authentication gateway | None |
| GET | `/api/settings/typography` | Returns typography and text direction settings | None |
| POST | `/api/settings/typography/<font_set>/<size>/<direction>` | Sets typography settings (font, size, direction) | **Write** |
| POST | `/api/settings/typography-feature/<font_name>/<feature>/<new_value>` | Sets individual font feature values | **Write** |

---

## Network Status (`/api/net`)

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| GET | `/api/net/status` | Returns network availability state | None |
| POST | `/api/net/enable` | Enables network mode | None |
| POST | `/api/net/disable` | Disables network mode | None |

---

## Debug (`/api/debug`)

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| GET | `/api/debug/status` | Returns debug mode state | None |
| GET | `/api/debug/enable` | Enables debug mode | None |
| GET | `/api/debug/disable` | Disables debug mode | None |

---

## Navigation (`/api/navigation`)

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| GET | `/api/navigation/bcv` | Returns current Book:Chapter:Verse position | None |
| POST | `/api/navigation/bcv/<book_code>/<chapter>/<verse>` | Sets BCV position (single verse) | **Write** |
| POST | `/api/navigation/bcv/<book_code>/<chapter>/<verse>/<to_verse>` | Sets BCV position with verse range | **Write** |

---

## App State (`/api/app-state`)

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| GET | `/api/app-state/current-project` | Returns current project identifier | None |
| POST | `/api/app-state/current-project/<source>/<organization>/<project>` | Sets current project | None |
| POST | `/api/app-state/current-project` | Clears current project | None |

---

## Temporary Files (`/api/temp`)

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| POST | `/api/temp/bytes` | Writes uploaded file to temp directory, returns UUID | **Write** |
| GET | `/api/temp/bytes/<temp_id>` | Reads temp file by UUID as binary | Read |

---

## Internationalization (`/api/i18n`)

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| GET | `/api/i18n/raw` | Returns raw nested i18n.json | Read |
| GET | `/api/i18n/negotiated/<filter..>` | Returns i18n with best language match | Read |
| GET | `/api/i18n/flat/<filter..>` | Returns flat i18n with colon-separated keys | Read |
| GET | `/api/i18n/untranslated/<lang>` | Returns untranslated terms in a language | Read |
| GET | `/api/i18n/used-languages` | Lists languages with translations | Read |
| POST | `/api/i18n/` | Replaces entire i18n.json file | **Write** |

---

## Git Operations (`/api/git`)

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| GET | `/api/git/list-local-repos` | Lists all local repos | Read |
| GET | `/api/git/status/<repo_path..>` | Returns git status (modified files) | Read |
| GET | `/api/git/log/<repo_path..>` | Returns commit log for repo | Read |
| GET | `/api/git/branches/<repo_path..>` | Lists branches in repo | Read |
| GET | `/api/git/remotes/<repo_path..>` | Lists git remotes for repo | Read |
| POST | `/api/git/clone-repo/<repo_path..>?<branch>` | Clones remote repo locally | **Write** |
| POST | `/api/git/delete/<repo_path..>` | Deletes a local repo | **Delete** |
| POST | `/api/git/branch/<branch_ref>/<repo_path..>` | Checks out existing branch | **Write** |
| POST | `/api/git/new-branch/<branch_ref>/<repo_path..>` | Creates and checks out new branch | **Write** |
| POST | `/api/git/remote/add/<repo_path..>?<remote_name>&<remote_url>` | Adds a git remote | **Write** |
| POST | `/api/git/remote/delete/<repo_path..>?<remote_name>` | Removes a git remote | **Write** |
| POST | `/api/git/push/<repo_path..>` | Pushes commits to remote | Remote |
| POST | `/api/git/pull-repo/<remote_name>/<repo_path..>` | Pulls commits from remote with merge | **Write** |
| POST | `/api/git/add-and-commit/<repo_path..>` | Stages and commits all changes | **Write** |
| POST | `/api/git/copy/<repo_path..>?<target_path>&<delete_src>&<add_ignore>` | Copies repo to new location | **Write** |
| POST | `/api/git/new-text-translation` | Creates new textTranslation repo | **Write** |
| POST | `/api/git/new-bcv-resource` | Creates new BCV resource repo | **Write** |
| POST | `/api/git/new-obs-resource` | Creates new OBS resource repo | **Write** |
| POST | `/api/git/new-scripture-book` | Adds book to scripture text translation | **Write** |
| POST | `/api/git/new-bcv-resource-book` | Adds book to BCV resource | **Write** |
| POST | `/api/git/new-tcore-resource` | Creates new Translation Core resource | **Write** |
| POST | `/api/git/new-translation-plan-resource` | Creates new translation plan resource | **Write** |

---

## Gitea Integration (`/api/gitea`)

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| GET | `/api/gitea/endpoints` | Returns configured Gitea gateway endpoints | None |
| GET | `/api/gitea/remote-repos/<gitea_server>/<gitea_org>` | Lists repos in remote Gitea org | Remote |
| GET | `/api/gitea/user-remote-repos/<gitea_server>/<gitea_user>` | Lists repos of remote Gitea user | Remote |
| GET | `/api/gitea/login/<token_key>/<redir_path..>` | Initiates auth with remote Gitea server | Remote |
| GET | `/api/gitea/logout/<token_key>` | Logs out from remote Gitea server | Remote |
| GET | `/api/gitea/my-collaborators/<proxy>/<organization>/<project>` | Returns project collaborators from remote | Remote |

---

## Burrito Content Operations (`/api/burrito`)

### Reading content

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| GET | `/api/burrito/ingredient/raw/<repo_path..>?<ipath>` | Returns text ingredient as plain text | Read |
| GET | `/api/burrito/ingredient/bytes/<repo_path..>?<ipath>` | Returns ingredient as binary | Read |
| GET | `/api/burrito/ingredients/raw/<repo_path..>?<ipath>` | Returns multiple text ingredients | Read |
| GET | `/api/burrito/ingredient/zipped/<repo_path..>?<ipath>` | Returns ingredient as zip file | Read |
| GET | `/api/burrito/metadata/raw/<repo_path..>` | Returns raw metadata.json | Read |
| GET | `/api/burrito/metadata/summary/<repo_path..>` | Returns flattened metadata summary | Read |
| GET | `/api/burrito/metadata/summaries?<org>` | Returns metadata for all/filtered repos | Read |
| GET | `/api/burrito/paths/<repo_path..>` | Lists files in ingredient directory | Read |
| GET | `/api/burrito/audit/<repo_path..>` | Validates burrito structure and metadata | Read |
| GET | `/api/burrito/zipped/<repo_path..>` | Returns entire repo as zip file | Read |

### Writing/modifying content

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| POST | `/api/burrito/ingredient/raw/<repo_path..>` | Writes/updates text ingredient | **Write** |
| POST | `/api/burrito/ingredient/bytes/<repo_path..>` | Writes/updates binary ingredient | **Write** |
| POST | `/api/burrito/ingredient/zipped/<repo_path..>` | Writes/updates ingredient from zip | **Write** |
| POST | `/api/burrito/ingredient/copy/<repo_path..>?<src_path>&<target_path>&<delete_src>` | Copies ingredient to new location | **Write** |
| POST | `/api/burrito/ingredient/revert/<repo_path..>?<ipath>` | Reverts ingredient from git | **Write** |
| POST | `/api/burrito/metadata/remake-ingredients/<repo_path..>` | Regenerates metadata for ingredients | **Write** |
| POST | `/api/burrito/zipped/<repo_path..>` | Imports repo from zip file | **Write** |
| POST | `/api/burrito/remake_burrito_from_zip/<temp_id>/<repo_path..>` | Recreates burrito from zip in temp storage | **Write** |

### Deleting content

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| POST | `/api/burrito/ingredient/delete/<repo_path..>?<ipath>` | Deletes single ingredient | **Delete** |
| POST | `/api/burrito/ingredients/delete/<repo_path..>?<ipath>` | Deletes multiple ingredients | **Delete** |

---

## Content Utilities (`/api/content-utils`)

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| GET | `/api/content-utils/templates` | Lists available content templates | Read |
| GET | `/api/content-utils/template/<template_name>/<filename>` | Returns content template file as text | Read |
| GET | `/api/content-utils/metadata-template/<template_name>` | Returns metadata template as JSON | Read |
| GET | `/api/content-utils/template-filenames/<template>` | Lists files in template directory | Read |
| GET | `/api/content-utils/versifications` | Lists available versification schemes | Read |
| GET | `/api/content-utils/versification/<versification_name>` | Returns versification scheme details | Read |
| GET | `/api/content-utils/product?<resource_path>` | Serves product-specific resources | Read |

---

## LLM (`/api/llm`)

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| GET | `/api/llm/model` | Lists available LLM models in local blobs | Read |
| POST | `/api/llm/rag-prompt` | Executes RAG prompt against LLM | None |

---

## Video Processing (`/api/video`)

| Method | Path | Description | Filesystem |
|--------|------|-------------|------------|
| POST | `/api/video/obs-para/<repo_path..>` | Generates video for OBS paragraph | **Write** |
| POST | `/api/video/obs-story/<repo_path..>` | Generates video for OBS story | **Write** |

---

## Summary

| Category | Read | Write | Delete | Remote | None |
|----------|------|-------|--------|--------|------|
| Root & Clients | 5 | 0 | 0 | 0 | 1 |
| Settings | 0 | 3 | 0 | 0 | 3 |
| Net/Debug | 0 | 0 | 0 | 0 | 6 |
| Navigation | 0 | 2 | 0 | 0 | 1 |
| App State | 0 | 0 | 0 | 0 | 3 |
| Temp Files | 1 | 1 | 0 | 0 | 0 |
| i18n | 5 | 1 | 0 | 0 | 0 |
| Git | 5 | 13 | 1 | 1 | 0 |
| Gitea | 0 | 0 | 0 | 4 | 2 |
| Burrito | 10 | 8 | 2 | 0 | 0 |
| Content Utils | 7 | 0 | 0 | 0 | 0 |
| LLM | 1 | 0 | 0 | 0 | 1 |
| Video | 0 | 2 | 0 | 0 | 0 |
| SSE | 0 | 0 | 0 | 0 | 1 |
| **Total** | **34** | **30** | **3** | **5** | **18** |

**Endpoints that modify the local filesystem (Write/Delete): 33 total** — primarily in the Git operations (14), Burrito content operations (10), and Settings (3) categories.
