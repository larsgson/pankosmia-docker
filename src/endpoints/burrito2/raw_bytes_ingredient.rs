use crate::auth::session::read_session;
use crate::auth::{GithubAppAuth, GithubClient};
use crate::catalog::CatalogRegistry;
use crate::gitea::github_read;
use crate::gitea::{resolve_read_source, CuratedOrgs, GiteaProxyClient, ReadSource};
use crate::identity::UserId;
use crate::store::sqlite_user_state::SqliteUserState;
use crate::store::SharedProjectStore;
use crate::structs::{AppSettings, BytesOrError};
use crate::utils::json_responses::make_bad_json_data_response;
use crate::utils::mime::mime_types;
use crate::utils::paths::{check_path_components, check_path_string_components, os_slash_str};
use rocket::http::{ContentType, CookieJar, Status};
use rocket::response::status;
use rocket::{get, State};
use std::path::{Components, PathBuf};
use std::sync::Arc;

#[get("/ingredient/bytes/<repo_path..>?<ipath>")]
pub async fn raw_bytes_ingredient(
    _state: &State<AppSettings>,
    store: &State<SharedProjectStore>,
    curated: &State<CuratedOrgs>,
    client: &State<GiteaProxyClient>,
    catalog: &State<Arc<CatalogRegistry>>,
    app_auth: &State<Option<GithubAppAuth>>,
    github_client: &State<GithubClient>,
    db: &State<Option<Arc<SqliteUserState>>>,
    cookies: &CookieJar<'_>,
    repo_path: PathBuf,
    ipath: String,
) -> status::Custom<(ContentType, BytesOrError)> {
    if !check_path_string_components(ipath.clone()) {
        return status::Custom(
            Status::BadRequest,
            (
                ContentType::JSON,
                BytesOrError::Error(make_bad_json_data_response("bad repo path".to_string())),
            ),
        );
    }

    match resolve_read_source(curated, &repo_path) {
        ReadSource::Github(parsed) => {
            let app_auth = match app_auth.inner().as_ref() {
                Some(a) => a,
                None => {
                    return status::Custom(
                        Status::ServiceUnavailable,
                        (
                            ContentType::JSON,
                            BytesOrError::Error(make_bad_json_data_response(
                                "GitHub App auth not configured".into(),
                            )),
                        ),
                    )
                }
            };
            let login = read_session(cookies).and_then(|uid| {
                db.inner().as_ref().and_then(|db| {
                    db.get_github_login(&UserId::from_github_id(uid))
                        .ok()
                        .flatten()
                })
            });
            let branch = github_read::resolve_branch(
                &parsed,
                login.as_deref(),
                catalog.inner(),
                app_auth,
                github_client.inner(),
            )
            .await
            .unwrap_or_else(|_| "main".to_string());
            let gh_ipath = format!("ingredients/{}", ipath);
            match github_read::fetch_file(
                &parsed,
                &gh_ipath,
                &branch,
                catalog.inner(),
                app_auth,
                github_client.inner(),
            )
            .await
            {
                Ok(Some(bytes)) => {
                    let mut split_ipath = ipath.split('.');
                    let mut suffix = "unknown";
                    if let Some(_) = split_ipath.next() {
                        if let Some(second) = split_ipath.next() {
                            suffix = second;
                        }
                    }
                    status::Custom(
                        Status::Ok,
                        (
                            match mime_types().get(suffix) {
                                Some(t) => t.clone(),
                                None => ContentType::new("application", "octet-stream"),
                            },
                            BytesOrError::Bytes(bytes),
                        ),
                    )
                }
                Ok(None) => status::Custom(
                    Status::NotFound,
                    (
                        ContentType::JSON,
                        BytesOrError::Error(make_bad_json_data_response(
                            "ingredient not found".into(),
                        )),
                    ),
                ),
                Err(e) => status::Custom(
                    Status::BadGateway,
                    (
                        ContentType::JSON,
                        BytesOrError::Error(make_bad_json_data_response(format!(
                            "github proxy: {}",
                            e
                        ))),
                    ),
                ),
            }
        }
        ReadSource::Gitea(parsed) => {
            match client
                .fetch_raw(&parsed.server, &parsed.org, &parsed.repo, &ipath, "master")
                .await
            {
                Ok((_content_type, bytes)) => {
                    let mut split_ipath = ipath.split('.');
                    let mut suffix = "unknown";
                    if let Some(_) = split_ipath.next() {
                        if let Some(second) = split_ipath.next() {
                            suffix = second;
                        }
                    }
                    status::Custom(
                        Status::Ok,
                        (
                            match mime_types().get(suffix) {
                                Some(t) => t.clone(),
                                None => ContentType::new("application", "octet-stream"),
                            },
                            BytesOrError::Bytes(bytes),
                        ),
                    )
                }
                Err(e) => status::Custom(
                    Status::BadGateway,
                    (
                        ContentType::JSON,
                        BytesOrError::Error(make_bad_json_data_response(format!(
                            "gitea proxy: {}",
                            e
                        ))),
                    ),
                ),
            }
        }
        ReadSource::LocalFilesystem => {
            let path_components: Components<'_> = repo_path.components();
            if !check_path_components(&mut path_components.clone()) {
                return status::Custom(
                    Status::BadRequest,
                    (
                        ContentType::JSON,
                        BytesOrError::Error(make_bad_json_data_response(
                            "bad repo path".to_string(),
                        )),
                    ),
                );
            }
            let path_to_serve = store.workspace_root().to_string_lossy().into_owned()
                + os_slash_str()
                + &repo_path.display().to_string()
                + "/ingredients/"
                + ipath.as_str();
            match std::fs::read(path_to_serve) {
                Ok(v) => {
                    let mut split_ipath = ipath.split('.');
                    let mut suffix = "unknown";
                    if let Some(_) = split_ipath.next() {
                        if let Some(second) = split_ipath.next() {
                            suffix = second;
                        }
                    }
                    status::Custom(
                        Status::Ok,
                        (
                            match mime_types().get(suffix) {
                                Some(t) => t.clone(),
                                None => ContentType::new("application", "octet-stream"),
                            },
                            BytesOrError::Bytes(v),
                        ),
                    )
                }
                Err(e) => status::Custom(
                    Status::BadRequest,
                    (
                        ContentType::JSON,
                        BytesOrError::Error(make_bad_json_data_response(
                            format!("could not read ingredient content: {}", e).to_string(),
                        )),
                    ),
                ),
            }
        }
    }
}
