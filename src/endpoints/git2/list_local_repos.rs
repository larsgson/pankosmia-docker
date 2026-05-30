use crate::auth::session::read_session;
use crate::gitea::{CuratedOrgs, GiteaProxyClient};
use crate::identity::UserId;
use crate::store::sqlite_user_state::SqliteUserState;
use crate::store::SharedProjectStore;
use crate::utils::response::ok_json_response;
use rocket::http::{ContentType, CookieJar};
use rocket::response::status;
use rocket::{get, State};
use std::sync::Arc;

#[get("/list-local-repos")]
pub async fn list_local_repos(
    _store: &State<SharedProjectStore>,
    _curated: &State<CuratedOrgs>,
    _client: &State<GiteaProxyClient>,
    db: &State<Option<Arc<SqliteUserState>>>,
    cookies: &CookieJar<'_>,
) -> status::Custom<(ContentType, String)> {
    let selected = read_session(cookies)
        .and_then(|uid| {
            db.inner().as_ref().and_then(|db| {
                let user_id = UserId::from_github_id(uid);
                db.get_selected_resources(&user_id).ok()
            })
        })
        .unwrap_or_default();

    ok_json_response(serde_json::to_string(&selected).unwrap())
}
