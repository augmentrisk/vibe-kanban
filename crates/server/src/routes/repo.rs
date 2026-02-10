use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
};
use db::models::{
    project::SearchResult,
    repo::{Repo, UpdateRepo},
};
use deployment::Deployment;
use serde::{Deserialize, Serialize};
use services::services::{file_search::SearchQuery, git::{GitBranch, GitCli}};
use ts_rs::TS;
use utils::response::ApiResponse;
use uuid::Uuid;

use crate::{DeploymentImpl, error::ApiError};

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct RegisterRepoRequest {
    pub path: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct InitRepoRequest {
    pub parent_path: String,
    pub folder_name: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct BatchRepoRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CloneRepoRequest {
    /// The git URL to clone from (HTTPS or SSH)
    pub url: String,
    /// Optional display name for the repository
    pub display_name: Option<String>,
}

pub async fn register_repo(
    State(deployment): State<DeploymentImpl>,
    ResponseJson(payload): ResponseJson<RegisterRepoRequest>,
) -> Result<ResponseJson<ApiResponse<Repo>>, ApiError> {
    let repo = deployment
        .repo()
        .register(
            &deployment.db().pool,
            &payload.path,
            payload.display_name.as_deref(),
        )
        .await?;

    Ok(ResponseJson(ApiResponse::success(repo)))
}

pub async fn init_repo(
    State(deployment): State<DeploymentImpl>,
    ResponseJson(payload): ResponseJson<InitRepoRequest>,
) -> Result<ResponseJson<ApiResponse<Repo>>, ApiError> {
    let repo = deployment
        .repo()
        .init_repo(
            &deployment.db().pool,
            deployment.git(),
            &payload.parent_path,
            &payload.folder_name,
        )
        .await?;

    Ok(ResponseJson(ApiResponse::success(repo)))
}

pub async fn clone_repo(
    State(deployment): State<DeploymentImpl>,
    ResponseJson(payload): ResponseJson<CloneRepoRequest>,
) -> Result<ResponseJson<ApiResponse<Repo>>, ApiError> {
    let repo = deployment
        .repo()
        .clone_repo(
            &deployment.db().pool,
            &payload.url,
            payload.display_name.as_deref(),
        )
        .await?;

    deployment
        .track_if_analytics_allowed(
            "repo_cloned",
            serde_json::json!({
                "repo_id": repo.id.to_string(),
            }),
        )
        .await;

    Ok(ResponseJson(ApiResponse::success(repo)))
}

pub async fn get_repo_branches(
    State(deployment): State<DeploymentImpl>,
    Path(repo_id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<Vec<GitBranch>>>, ApiError> {
    let repo = deployment
        .repo()
        .get_by_id(&deployment.db().pool, repo_id)
        .await?;

    let branches = deployment.git().get_all_branches(&repo.path)?;
    Ok(ResponseJson(ApiResponse::success(branches)))
}

pub async fn get_repos_batch(
    State(deployment): State<DeploymentImpl>,
    ResponseJson(payload): ResponseJson<BatchRepoRequest>,
) -> Result<ResponseJson<ApiResponse<Vec<Repo>>>, ApiError> {
    let repos = Repo::find_by_ids(&deployment.db().pool, &payload.ids).await?;
    Ok(ResponseJson(ApiResponse::success(repos)))
}

pub async fn get_repos(
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<Vec<Repo>>>, ApiError> {
    let repos = Repo::list_all(&deployment.db().pool).await?;
    Ok(ResponseJson(ApiResponse::success(repos)))
}

pub async fn get_repo(
    State(deployment): State<DeploymentImpl>,
    Path(repo_id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<Repo>>, ApiError> {
    let repo = deployment
        .repo()
        .get_by_id(&deployment.db().pool, repo_id)
        .await?;
    Ok(ResponseJson(ApiResponse::success(repo)))
}

pub async fn update_repo(
    State(deployment): State<DeploymentImpl>,
    Path(repo_id): Path<Uuid>,
    ResponseJson(payload): ResponseJson<UpdateRepo>,
) -> Result<ResponseJson<ApiResponse<Repo>>, ApiError> {
    let repo = Repo::update(&deployment.db().pool, repo_id, &payload).await?;
    Ok(ResponseJson(ApiResponse::success(repo)))
}

pub async fn search_repo(
    State(deployment): State<DeploymentImpl>,
    Path(repo_id): Path<Uuid>,
    Query(search_query): Query<SearchQuery>,
) -> Result<ResponseJson<ApiResponse<Vec<SearchResult>>>, StatusCode> {
    if search_query.q.trim().is_empty() {
        return Ok(ResponseJson(ApiResponse::error(
            "Query parameter 'q' is required and cannot be empty",
        )));
    }

    let repo = match deployment
        .repo()
        .get_by_id(&deployment.db().pool, repo_id)
        .await
    {
        Ok(repo) => repo,
        Err(e) => {
            tracing::error!("Failed to get repo {}: {}", repo_id, e);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    match deployment
        .file_search_cache()
        .search_repo(&repo.path, &search_query.q, search_query.mode)
        .await
    {
        Ok(results) => Ok(ResponseJson(ApiResponse::success(results))),
        Err(e) => {
            tracing::error!("Failed to search files in repo {}: {}", repo_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MainBranchInfo {
    pub branch: String,
    pub sha: String,
    pub short_sha: String,
    pub subject: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PullMainResult {
    pub updated: bool,
    pub branch: String,
    pub sha: String,
    pub short_sha: String,
    pub subject: String,
}

pub async fn get_main_branch_info(
    State(deployment): State<DeploymentImpl>,
    Path(repo_id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<MainBranchInfo>>, ApiError> {
    let repo = deployment
        .repo()
        .get_by_id(&deployment.db().pool, repo_id)
        .await?;

    let target_branch = repo
        .default_target_branch
        .as_deref()
        .unwrap_or("main")
        .to_string();

    let git = deployment.git();
    let sha = git.get_branch_oid(&repo.path, &target_branch)?;
    let subject = git.get_commit_subject(&repo.path, &sha)?;
    let short_sha = sha.chars().take(7).collect::<String>();

    Ok(ResponseJson(ApiResponse::success(MainBranchInfo {
        branch: target_branch,
        sha,
        short_sha,
        subject,
    })))
}

pub async fn pull_main_branch(
    State(deployment): State<DeploymentImpl>,
    Path(repo_id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<PullMainResult>>, ApiError> {
    let repo = deployment
        .repo()
        .get_by_id(&deployment.db().pool, repo_id)
        .await?;

    let target_branch = repo
        .default_target_branch
        .as_deref()
        .unwrap_or("main")
        .to_string();

    let git_cli = GitCli::new();

    // Check if the repository has a remote origin
    let remote_url = git_cli
        .get_remote_url(&repo.path, "origin")
        .map_err(|_| ApiError::BadRequest("Repository has no origin remote".to_string()))?;

    // Fetch the target branch from origin
    let refspec = format!("refs/heads/{0}:refs/remotes/origin/{0}", target_branch);
    git_cli
        .fetch_with_refspec(&repo.path, &remote_url, &refspec)
        .map_err(|e| ApiError::BadRequest(format!("Failed to fetch from origin: {}", e)))?;

    // Get local and remote commits
    let local_sha = git_cli
        .rev_parse(&repo.path, &target_branch)
        .unwrap_or_default();
    let remote_sha = git_cli
        .rev_parse(&repo.path, &format!("origin/{}", target_branch))
        .map_err(|e| ApiError::BadRequest(format!("Failed to resolve remote branch: {}", e)))?;

    let updated = local_sha != remote_sha;
    if updated {
        // Fast-forward the local branch ref to match origin
        git_cli
            .update_ref(
                &repo.path,
                &format!("refs/heads/{}", target_branch),
                &remote_sha,
            )
            .map_err(|e| {
                ApiError::BadRequest(format!("Failed to update local branch: {}", e))
            })?;
    }

    let git = deployment.git();
    let final_sha = git.get_branch_oid(&repo.path, &target_branch)?;
    let subject = git.get_commit_subject(&repo.path, &final_sha)?;
    let short_sha = final_sha.chars().take(7).collect::<String>();

    Ok(ResponseJson(ApiResponse::success(PullMainResult {
        updated,
        branch: target_branch,
        sha: final_sha,
        short_sha,
        subject,
    })))
}

pub fn router() -> Router<DeploymentImpl> {
    Router::new()
        .route("/repos", get(get_repos).post(register_repo))
        .route("/repos/init", post(init_repo))
        .route("/repos/clone", post(clone_repo))
        .route("/repos/batch", post(get_repos_batch))
        .route("/repos/{repo_id}", get(get_repo).put(update_repo))
        .route("/repos/{repo_id}/branches", get(get_repo_branches))
        .route("/repos/{repo_id}/main-branch-info", get(get_main_branch_info))
        .route("/repos/{repo_id}/pull-main", post(pull_main_branch))
        .route("/repos/{repo_id}/search", get(search_repo))
}
