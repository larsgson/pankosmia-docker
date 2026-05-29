use crate::auth::{GithubAppAuth, GithubClient};
use crate::catalog::CatalogRegistry;
use crate::gitea::github_read;
use crate::gitea::{resolve_read_source, CuratedOrgs, GiteaProxyClient, ReadSource};
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
use std::path::{Components, PathBuf};
use std::sync::Arc;

#[get("/metadata/raw/<repo_path..>")]
pub async fn raw_metadata(
    _state: &State<AppSettings>,
    store: &State<SharedProjectStore>,
    curated: &State<CuratedOrgs>,
    client: &State<GiteaProxyClient>,
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
            let branch = github_read::default_branch(
                &parsed,
                catalog.inner(),
                app_auth,
                github_client.inner(),
            )
            .await
            .unwrap_or_else(|_| "main".to_string());
            match github_read::fetch_file(
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
                    Ok(json_str) => ok_json_response(json_str),
                    Err(e) => not_ok_json_response(
                        Status::BadGateway,
                        make_bad_json_data_response(format!("not valid UTF-8: {}", e)),
                    ),
                },
                Ok(None) => not_ok_json_response(
                    Status::NotFound,
                    make_bad_json_data_response("metadata.json not found".into()),
                ),
                Err(e) => not_ok_json_response(
                    Status::BadGateway,
                    make_bad_json_data_response(format!("github proxy: {}", e)),
                ),
            }
        }
        ReadSource::Gitea(parsed) => {
            match client
                .fetch_raw(
                    &parsed.server,
                    &parsed.org,
                    &parsed.repo,
                    "metadata.json",
                    "master",
                )
                .await
            {
                Ok((_ct, bytes)) => match String::from_utf8(bytes) {
                    Ok(json_str) => ok_json_response(json_str),
                    Err(e) => not_ok_json_response(
                        Status::BadGateway,
                        make_bad_json_data_response(format!("not valid UTF-8: {}", e)),
                    ),
                },
                Err(e) => not_ok_json_response(
                    Status::BadGateway,
                    make_bad_json_data_response(format!("gitea proxy: {}", e)),
                ),
            }
        }
        ReadSource::LocalFilesystem => {
            let path_components: Components<'_> = repo_path.components();
            if check_path_components(&mut path_components.clone()) {
                let path_to_serve = store.workspace_root().to_string_lossy().into_owned()
                    + os_slash_str()
                    + &repo_path.display().to_string()
                    + "/metadata.json";
                match std::fs::read_to_string(path_to_serve) {
                    Ok(v) => ok_json_response(v),
                    Err(e) => not_ok_json_response(
                        Status::BadRequest,
                        make_bad_json_data_response(format!("could not read metadata: {}", e)),
                    ),
                }
            } else {
                not_ok_bad_repo_json_response()
            }
        }
    }
}
