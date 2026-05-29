use crate::auth::{resolve_installation_id, GithubAppAuth, GithubClient};
use crate::catalog::CatalogRegistry;
use crate::gitea::ParsedRepoPath;
use std::sync::Arc;

pub struct GithubReadError(pub String);

impl std::fmt::Display for GithubReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

async fn resolve_token(
    parsed: &ParsedRepoPath,
    catalog: &Arc<CatalogRegistry>,
    app_auth: &GithubAppAuth,
) -> Result<(String, String), GithubReadError> {
    let repo_full = format!("{}/{}", parsed.org, parsed.repo);
    for entry in catalog.list() {
        if entry.repo == repo_full {
            let installation_id =
                resolve_installation_id(entry.installation_id, entry.code.as_str())
                    .map_err(|e| GithubReadError(format!("installation id: {}", e)))?;
            let token = app_auth
                .installation_token(installation_id)
                .await
                .map_err(|e| GithubReadError(format!("token: {}", e)))?;
            return Ok((token, repo_full));
        }
    }
    Err(GithubReadError(format!(
        "repo {}/{} not found in catalog",
        parsed.org, parsed.repo
    )))
}

pub async fn fetch_file(
    parsed: &ParsedRepoPath,
    path: &str,
    branch: &str,
    catalog: &Arc<CatalogRegistry>,
    app_auth: &GithubAppAuth,
    github_client: &GithubClient,
) -> Result<Option<Vec<u8>>, GithubReadError> {
    let (token, repo_full) = resolve_token(parsed, catalog, app_auth).await?;
    github_client
        .get_file_bytes(&token, &repo_full, path, branch)
        .await
        .map_err(|e| GithubReadError(format!("fetch {}: {}", path, e)))
}

pub async fn list_tree(
    parsed: &ParsedRepoPath,
    login: Option<&str>,
    catalog: &Arc<CatalogRegistry>,
    app_auth: &GithubAppAuth,
    github_client: &GithubClient,
) -> Result<Vec<String>, GithubReadError> {
    let (token, repo_full) = resolve_token(parsed, catalog, app_auth).await?;
    let branch = resolve_branch(parsed, login, catalog, app_auth, github_client)
        .await
        .unwrap_or_else(|_| "main".to_string());
    let commit_sha = github_client
        .get_branch_sha(&token, &repo_full, &branch)
        .await
        .map_err(|e| GithubReadError(format!("get branch sha: {}", e)))?
        .ok_or_else(|| GithubReadError(format!("branch {} not found", branch)))?;
    let tree_sha = github_client
        .get_commit_tree_sha(&token, &repo_full, &commit_sha)
        .await
        .map_err(|e| GithubReadError(format!("get tree sha: {}", e)))?;
    let (entries, _truncated) = github_client
        .get_tree_recursive(&token, &repo_full, &tree_sha)
        .await
        .map_err(|e| GithubReadError(format!("list tree: {}", e)))?;
    Ok(entries
        .into_iter()
        .filter(|e| e.entry_type == "blob")
        .map(|e| e.path)
        .collect())
}

pub async fn resolve_branch(
    parsed: &ParsedRepoPath,
    login: Option<&str>,
    catalog: &Arc<CatalogRegistry>,
    app_auth: &GithubAppAuth,
    github_client: &GithubClient,
) -> Result<String, GithubReadError> {
    let (token, repo_full) = resolve_token(parsed, catalog, app_auth).await?;
    if let Some(login) = login {
        let working = format!("pankosmia-edit-{}", login);
        if let Ok(Some(_)) = github_client
            .get_branch_sha(&token, &repo_full, &working)
            .await
        {
            return Ok(working);
        }
    }
    let repo = github_client
        .get_repo(&token, &repo_full)
        .await
        .map_err(|e| GithubReadError(format!("get repo: {}", e)))?;
    Ok(repo.default_branch.unwrap_or_else(|| "main".into()))
}
