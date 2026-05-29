use crate::auth::{GithubAppAuth, GithubClient};
use crate::catalog::CatalogRegistry;
use crate::gitea::github_read;
use crate::gitea::{resolve_read_source, CuratedOrgs, GiteaCache, GiteaProxyClient, ReadSource};
use crate::store::SharedProjectStore;
use crate::structs::AppSettings;
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::paths::{check_path_components, os_slash_str};
use crate::utils::response::{
    not_ok_bad_repo_json_response, not_ok_json_response, ok_json_response,
};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::{get, State};
use std::path::{Components, Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;

#[get("/paths/<repo_path..>")]
pub async fn get_repo_file_paths(
    _state: &State<AppSettings>,
    store: &State<SharedProjectStore>,
    curated: &State<CuratedOrgs>,
    client: &State<GiteaProxyClient>,
    cache: &State<GiteaCache>,
    catalog: &State<Arc<CatalogRegistry>>,
    app_auth: &State<Option<GithubAppAuth>>,
    github_client: &State<GithubClient>,
    repo_path: PathBuf,
) -> status::Custom<(ContentType, String)> {
    match resolve_read_source(curated, &repo_path) {
        ReadSource::Github(parsed) => {
            let app_auth = match app_auth.inner().as_ref() {
                Some(a) => a,
                None => {
                    return not_ok_json_response(
                        Status::ServiceUnavailable,
                        make_bad_json_data_response("GitHub App auth not configured".into()),
                    )
                }
            };
            match github_read::list_tree(&parsed, catalog.inner(), app_auth, github_client.inner())
                .await
            {
                Ok(all_paths) => {
                    let paths: Vec<String> = all_paths
                        .into_iter()
                        .filter_map(|p| p.strip_prefix("ingredients/").map(|s| s.to_string()))
                        .filter(|p| {
                            !p.starts_with('.') && !p.ends_with(".bak") && !p.contains("/.")
                        })
                        .collect();
                    ok_json_response(serde_json::to_string(&paths).unwrap())
                }
                Err(e) => not_ok_json_response(
                    Status::BadGateway,
                    make_bad_json_data_response(format!("github proxy: {}", e)),
                ),
            }
        }
        ReadSource::Gitea(parsed) => {
            let cache_key = format!("{}/{}/{}", parsed.server, parsed.org, parsed.repo);
            if let Some(cached) = cache.trees.get(&cache_key) {
                return ok_json_response(serde_json::to_string(cached.as_ref()).unwrap());
            }

            match client
                .list_tree(&parsed.server, &parsed.org, &parsed.repo, "master")
                .await
            {
                Ok(entries) => {
                    let paths: Vec<String> = entries
                        .iter()
                        .filter(|e| e.entry_type == "blob")
                        .filter_map(|e| e.path.strip_prefix("ingredients/").map(|s| s.to_string()))
                        .filter(|p| {
                            !p.starts_with('.') && !p.ends_with(".bak") && !p.contains("/.")
                        })
                        .collect();
                    cache.trees.insert(cache_key, Arc::new(paths.clone()));
                    ok_json_response(serde_json::to_string(&paths).unwrap())
                }
                Err(e) => not_ok_json_response(
                    Status::BadGateway,
                    make_bad_json_data_response(format!("gitea proxy: {}", e)),
                ),
            }
        }
        ReadSource::LocalFilesystem => {
            let path_components: Components<'_> = repo_path.components();
            if check_path_components(&mut path_components.clone()) {
                let full_repo_dir = format!(
                    "{}{}{}/ingredients/",
                    store.workspace_root().to_string_lossy().into_owned(),
                    os_slash_str(),
                    &repo_path.display().to_string()
                );
                if !Path::new(&full_repo_dir).exists() {
                    return ok_json_response("[]".to_string());
                }
                let mut paths = Vec::new();
                for entry in WalkDir::new(&full_repo_dir) {
                    let entry_string = match entry {
                        Ok(ent) => ent.path().display().to_string(),
                        Err(e) => {
                            return not_ok_json_response(
                                Status::BadRequest,
                                make_bad_json_data_response(format!("could not read entry: {}", e)),
                            );
                        }
                    };
                    if Path::new(&entry_string).is_file() {
                        let truncated_entry_string = entry_string.replace(&full_repo_dir, "");
                        if !truncated_entry_string.starts_with('.')
                            && !truncated_entry_string.ends_with(".bak")
                            && !truncated_entry_string
                                .contains(format!("{}.", os_slash_str()).as_str())
                        {
                            paths.push(truncated_entry_string.replace('\\', "/"));
                        }
                    }
                }
                ok_json_response(serde_json::to_string(&paths).unwrap())
            } else {
                not_ok_bad_repo_json_response()
            }
        }
    }
}
