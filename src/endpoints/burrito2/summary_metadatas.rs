use crate::auth::session::read_session;
use crate::auth::{GithubAppAuth, GithubClient};
use crate::catalog::CatalogRegistry;
use crate::gitea::github_read;
use crate::gitea::{CuratedOrgs, GiteaCache, GiteaProxyClient, ParsedRepoPath};
use crate::identity::UserId;
use crate::store::sqlite_user_state::SqliteUserState;
use crate::store::SharedProjectStore;
use crate::structs::AppSettings;
use crate::structs::MetadataSummary;
use crate::utils::burrito::{summary_metadata_from_file, summary_metadata_from_str};
use crate::utils::paths::os_slash_str;
use crate::utils::response::ok_json_response;
use rocket::http::{ContentType, CookieJar};
use rocket::response::status;
use rocket::{get, State};
use std::collections::BTreeMap;
use std::sync::Arc;

fn fallback_summary() -> MetadataSummary {
    MetadataSummary {
        name: "? Bad Metadata JSON ?".to_string(),
        description: "?".to_string(),
        abbreviation: "?".to_string(),
        generated_date: "?".to_string(),
        flavor_type: "?".to_string(),
        flavor: "?".to_string(),
        language_code: "?".to_string(),
        language_name: "?".to_string(),
        script_direction: "?".to_string(),
        book_codes: vec![],
        timestamp: 0,
    }
}

#[get("/metadata/summaries?<org>")]
pub async fn summary_metadatas(
    _state: &State<AppSettings>,
    store: &State<SharedProjectStore>,
    curated: &State<CuratedOrgs>,
    client: &State<GiteaProxyClient>,
    _cache: &State<GiteaCache>,
    catalog: &State<Arc<CatalogRegistry>>,
    app_auth: &State<Option<GithubAppAuth>>,
    github_client: &State<GithubClient>,
    db: &State<Option<Arc<SqliteUserState>>>,
    cookies: &CookieJar<'_>,
    #[allow(unused)] org: Option<String>,
) -> status::Custom<(ContentType, String)> {
    // Get the user's selected resources first — only fetch metadata for those.
    let (selected, login) = match read_session(cookies) {
        Some(uid) => match db.inner().as_ref() {
            Some(db_ref) => {
                let user_id = UserId::from_github_id(uid);
                let sel = db_ref.get_selected_resources(&user_id).unwrap_or_default();
                let login = db_ref.get_github_login(&user_id).ok().flatten();
                (sel, login)
            }
            None => (Vec::new(), None),
        },
        None => (Vec::new(), None),
    };

    if selected.is_empty() {
        return ok_json_response("{}".to_string());
    }

    let mut repos: BTreeMap<String, MetadataSummary> = BTreeMap::new();

    for path in &selected {
        if path.starts_with("github.com/") {
            // GitHub-hosted repo: fetch metadata via GitHub API
            let app_auth = match app_auth.inner().as_ref() {
                Some(a) => a,
                None => continue,
            };
            let parts: Vec<&str> = path.splitn(3, '/').collect();
            if parts.len() != 3 {
                continue;
            }
            let parsed = ParsedRepoPath {
                server: parts[0].to_string(),
                org: parts[1].to_string(),
                repo: parts[2].to_string(),
            };
            let branch = github_read::resolve_branch(
                &parsed,
                login.as_deref(),
                catalog.inner(),
                app_auth,
                github_client.inner(),
            )
            .await
            .unwrap_or_else(|_| "main".to_string());
            let summary = match github_read::fetch_file(
                &parsed,
                "metadata.json",
                &branch,
                catalog.inner(),
                app_auth,
                github_client.inner(),
            )
            .await
            {
                Ok(Some(bytes)) => match String::from_utf8(bytes) {
                    Ok(json_str) => {
                        summary_metadata_from_str(&json_str).unwrap_or_else(|_| fallback_summary())
                    }
                    Err(_) => fallback_summary(),
                },
                _ => fallback_summary(),
            };
            repos.insert(path.clone(), summary);
        } else {
            // Curated org or local: check Gitea cache, then fetch, then local FS
            let parts: Vec<&str> = path.splitn(3, '/').collect();
            if parts.len() != 3 {
                continue;
            }
            let server_org = format!("{}/{}", parts[0], parts[1]);

            if curated.is_curated(&server_org) {
                // Try Gitea proxy
                match client
                    .fetch_raw(parts[0], parts[1], parts[2], "metadata.json", "master")
                    .await
                {
                    Ok((_ct, bytes)) => {
                        if let Ok(json_str) = String::from_utf8(bytes) {
                            let summary = summary_metadata_from_str(&json_str)
                                .unwrap_or_else(|_| fallback_summary());
                            repos.insert(path.clone(), summary);
                        }
                    }
                    Err(_) => {
                        repos.insert(path.clone(), fallback_summary());
                    }
                }
            } else {
                // Local filesystem
                let metadata_path = format!(
                    "{}{}{}{}metadata.json",
                    store.workspace_root().to_string_lossy(),
                    os_slash_str(),
                    path,
                    os_slash_str()
                );
                let summary = summary_metadata_from_file(metadata_path)
                    .unwrap_or_else(|_| fallback_summary());
                repos.insert(path.clone(), summary);
            }
        }
    }

    ok_json_response(serde_json::to_string(&repos).unwrap())
}
