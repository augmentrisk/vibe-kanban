use std::{path::Path, time::Duration};

use db::{DBService, models::repo::Repo};
use thiserror::Error;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::services::git::{GitCli, GitCliError};

#[derive(Debug, Error)]
enum MainBranchSyncError {
    #[error(transparent)]
    GitCli(#[from] GitCliError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("Repository path does not exist: {0}")]
    RepoPathNotFound(String),
}

/// Service to regularly sync main branches from remote origins
pub struct MainBranchSyncService {
    db: DBService,
    poll_interval: Duration,
}

impl MainBranchSyncService {
    pub async fn spawn(db: DBService) -> tokio::task::JoinHandle<()> {
        let service = Self {
            db,
            poll_interval: Duration::from_secs(5 * 60), // Poll every 5 minutes
        };
        tokio::spawn(async move {
            service.start().await;
        })
    }

    async fn start(&self) {
        info!(
            "Starting main branch sync service with interval {:?}",
            self.poll_interval
        );

        let mut interval = interval(self.poll_interval);

        loop {
            interval.tick().await;
            if let Err(e) = self.sync_all_repos().await {
                error!("Error syncing repositories: {}", e);
            }
        }
    }

    /// Sync all repositories by pulling main from their remotes
    async fn sync_all_repos(&self) -> Result<(), MainBranchSyncError> {
        let repos = Repo::list_all(&self.db.pool).await?;

        if repos.is_empty() {
            debug!("No repositories to sync");
            return Ok(());
        }

        debug!("Syncing {} repositories", repos.len());

        for repo in repos {
            if let Err(e) = self.sync_repo(&repo).await {
                // Log errors but continue with other repos
                error!(
                    "Error syncing repository {} ({}): {}",
                    repo.display_name,
                    repo.path.display(),
                    e
                );
            }
        }

        Ok(())
    }

    /// Sync a single repository by pulling from origin
    async fn sync_repo(&self, repo: &Repo) -> Result<(), MainBranchSyncError> {
        let repo_path = &repo.path;

        // Verify the repository path exists
        if !repo_path.exists() {
            return Err(MainBranchSyncError::RepoPathNotFound(
                repo_path.display().to_string(),
            ));
        }

        let git = GitCli::new();

        // Check if the repository has a remote origin
        let remote_url = match git.get_remote_url(repo_path, "origin") {
            Ok(url) => url,
            Err(GitCliError::CommandFailed(_)) => {
                // No origin remote, skip this repo silently
                debug!(
                    "Repository {} has no origin remote, skipping sync",
                    repo.display_name
                );
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };

        // Determine which branch to sync (use default_target_branch or "main")
        let target_branch = repo
            .default_target_branch
            .as_deref()
            .unwrap_or("main")
            .to_string();

        debug!(
            "Syncing repository {} (branch: {}, remote: {})",
            repo.display_name, target_branch, remote_url
        );

        // Fetch the target branch from origin
        let refspec = format!("refs/heads/{0}:refs/remotes/origin/{0}", target_branch);
        match git.fetch_with_refspec(repo_path, &remote_url, &refspec) {
            Ok(_) => {
                debug!(
                    "Fetched {} from origin for {}",
                    target_branch, repo.display_name
                );
            }
            Err(e) => {
                warn!(
                    "Failed to fetch {} from origin for {}: {}",
                    target_branch, repo.display_name, e
                );
                return Err(e.into());
            }
        }

        // Try to fast-forward the local branch to match origin
        // We need to check if the local branch exists and update it
        match self
            .fast_forward_local_branch(repo_path, &target_branch)
            .await
        {
            Ok(true) => {
                info!(
                    "Updated {} branch for repository {} from origin",
                    target_branch, repo.display_name
                );
            }
            Ok(false) => {
                // Already up to date, don't log (this is the common case)
                debug!(
                    "Repository {} branch {} is already up to date",
                    repo.display_name, target_branch
                );
            }
            Err(e) => {
                warn!(
                    "Failed to fast-forward {} for {}: {}",
                    target_branch, repo.display_name, e
                );
                return Err(e);
            }
        }

        Ok(())
    }

    /// Fast-forward the local branch to match origin, returns true if updated
    async fn fast_forward_local_branch(
        &self,
        repo_path: &Path,
        branch: &str,
    ) -> Result<bool, MainBranchSyncError> {
        let git = GitCli::new();

        // Get the current commit for the local branch
        let local_commit = match self.get_branch_commit(repo_path, branch) {
            Ok(commit) => commit,
            Err(e) => {
                debug!("Could not get local commit for branch {}: {}", branch, e);
                return Ok(false);
            }
        };

        // Get the commit for the remote tracking branch
        let remote_branch = format!("origin/{}", branch);
        let remote_commit = match self.get_branch_commit(repo_path, &remote_branch) {
            Ok(commit) => commit,
            Err(e) => {
                debug!(
                    "Could not get remote commit for branch {}: {}",
                    remote_branch, e
                );
                return Ok(false);
            }
        };

        // If they're the same, we're already up to date
        if local_commit == remote_commit {
            return Ok(false);
        }

        // Update the local branch to point to the remote commit
        // Using `git update-ref` is safe for the canonical repo since it doesn't have a working tree
        git.update_ref(repo_path, &format!("refs/heads/{}", branch), &remote_commit)?;

        Ok(true)
    }

    /// Get the commit SHA for a branch reference
    fn get_branch_commit(&self, repo_path: &Path, branch: &str) -> Result<String, GitCliError> {
        let git = GitCli::new();
        git.rev_parse(repo_path, branch)
    }
}
