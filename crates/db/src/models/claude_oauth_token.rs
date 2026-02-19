use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ClaudeOAuthTokenError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("Token not found")]
    NotFound,
    #[error("No tokens available for rotation")]
    NoTokensAvailable,
}

/// Obfuscate a sensitive string for logging (show first 8 and last 4 chars)
fn obfuscate_token(token: &str) -> String {
    if token.len() <= 12 {
        return "[REDACTED]".to_string();
    }
    let prefix = &token[..8];
    let suffix = &token[token.len() - 4..];
    format!("{}...{}", prefix, suffix)
}

/// Stored Claude Code OAuth token for a user
#[derive(Clone, FromRow, Serialize, Deserialize)]
pub struct ClaudeOAuthToken {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(skip_serializing)] // Never expose encrypted token to frontend
    pub encrypted_token: String,
    pub token_hint: Option<String>,
    #[sqlx(rename = "created_at")]
    pub created_at: DateTime<Utc>,
    #[sqlx(rename = "expires_at")]
    pub expires_at: Option<DateTime<Utc>>,
    #[sqlx(rename = "last_used_at")]
    pub last_used_at: Option<DateTime<Utc>>,
}

// Custom Debug implementation to obfuscate the encrypted_token field
impl fmt::Debug for ClaudeOAuthToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClaudeOAuthToken")
            .field("id", &self.id)
            .field("user_id", &self.user_id)
            .field("encrypted_token", &obfuscate_token(&self.encrypted_token))
            .field("token_hint", &self.token_hint)
            .field("created_at", &self.created_at)
            .field("expires_at", &self.expires_at)
            .field("last_used_at", &self.last_used_at)
            .finish()
    }
}

/// Token status for frontend display (no sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ClaudeOAuthTokenStatus {
    pub has_token: bool,
    pub token_hint: Option<String>,
    #[ts(type = "Date | null")]
    pub created_at: Option<DateTime<Utc>>,
    #[ts(type = "Date | null")]
    pub expires_at: Option<DateTime<Utc>>,
    #[ts(type = "Date | null")]
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_expired: bool,
}

/// Combined user and token status for admin view
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UserTokenStatus {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub token_status: ClaudeOAuthTokenStatus,
}

impl ClaudeOAuthToken {
    /// Find token by user ID
    pub async fn find_by_user_id(
        pool: &SqlitePool,
        user_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            ClaudeOAuthToken,
            r#"SELECT id as "id!: Uuid",
                      user_id as "user_id!: Uuid",
                      encrypted_token,
                      token_hint,
                      created_at as "created_at!: DateTime<Utc>",
                      expires_at as "expires_at: DateTime<Utc>",
                      last_used_at as "last_used_at: DateTime<Utc>"
               FROM claude_oauth_tokens
               WHERE user_id = $1"#,
            user_id
        )
        .fetch_optional(pool)
        .await
    }

    /// Create or update token for user
    pub async fn upsert(
        pool: &SqlitePool,
        user_id: Uuid,
        encrypted_token: &str,
        token_hint: Option<&str>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as!(
            ClaudeOAuthToken,
            r#"INSERT INTO claude_oauth_tokens (id, user_id, encrypted_token, token_hint, expires_at)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT(user_id) DO UPDATE SET
                   encrypted_token = excluded.encrypted_token,
                   token_hint = excluded.token_hint,
                   expires_at = excluded.expires_at,
                   created_at = datetime('now', 'subsec'),
                   last_used_at = NULL
               RETURNING id as "id!: Uuid",
                         user_id as "user_id!: Uuid",
                         encrypted_token,
                         token_hint,
                         created_at as "created_at!: DateTime<Utc>",
                         expires_at as "expires_at: DateTime<Utc>",
                         last_used_at as "last_used_at: DateTime<Utc>""#,
            id,
            user_id,
            encrypted_token,
            token_hint,
            expires_at,
        )
        .fetch_one(pool)
        .await
    }

    /// Get next token using round-robin rotation (least recently used first)
    /// Skips expired tokens
    pub async fn get_next_for_rotation(
        pool: &SqlitePool,
    ) -> Result<Option<Self>, ClaudeOAuthTokenError> {
        let token = sqlx::query_as!(
            ClaudeOAuthToken,
            r#"SELECT id as "id!: Uuid",
                      user_id as "user_id!: Uuid",
                      encrypted_token,
                      token_hint,
                      created_at as "created_at!: DateTime<Utc>",
                      expires_at as "expires_at: DateTime<Utc>",
                      last_used_at as "last_used_at: DateTime<Utc>"
               FROM claude_oauth_tokens
               WHERE expires_at IS NULL OR expires_at > datetime('now')
               ORDER BY last_used_at ASC NULLS FIRST, created_at ASC
               LIMIT 1"#
        )
        .fetch_optional(pool)
        .await?;

        Ok(token)
    }

    /// Update last_used_at timestamp for a token
    pub async fn mark_used(pool: &SqlitePool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE claude_oauth_tokens
               SET last_used_at = datetime('now', 'subsec')
               WHERE id = $1"#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete token for user
    pub async fn delete_for_user(pool: &SqlitePool, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"DELETE FROM claude_oauth_tokens WHERE user_id = $1"#,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get all valid (non-expired) tokens
    pub async fn find_all_valid(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            ClaudeOAuthToken,
            r#"SELECT id as "id!: Uuid",
                      user_id as "user_id!: Uuid",
                      encrypted_token,
                      token_hint,
                      created_at as "created_at!: DateTime<Utc>",
                      expires_at as "expires_at: DateTime<Utc>",
                      last_used_at as "last_used_at: DateTime<Utc>"
               FROM claude_oauth_tokens
               WHERE expires_at IS NULL OR expires_at > datetime('now')
               ORDER BY created_at ASC"#
        )
        .fetch_all(pool)
        .await
    }

    /// Count available (non-expired) tokens
    pub async fn count_available(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!: i64"
               FROM claude_oauth_tokens
               WHERE expires_at IS NULL OR expires_at > datetime('now')"#
        )
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// Convert to status for frontend display
    pub fn to_status(&self) -> ClaudeOAuthTokenStatus {
        let is_expired = self.expires_at.map(|exp| exp < Utc::now()).unwrap_or(false);

        ClaudeOAuthTokenStatus {
            has_token: true,
            token_hint: self.token_hint.clone(),
            created_at: Some(self.created_at),
            expires_at: self.expires_at,
            last_used_at: self.last_used_at,
            is_expired,
        }
    }
}

impl ClaudeOAuthTokenStatus {
    /// Create a "no token" status
    pub fn no_token() -> Self {
        Self {
            has_token: false,
            token_hint: None,
            created_at: None,
            expires_at: None,
            last_used_at: None,
            is_expired: false,
        }
    }
}

/// Generate a token hint from a raw token (last 4 characters)
pub fn generate_token_hint(token: &str) -> String {
    let hint_chars: String = token.chars().rev().take(4).collect();
    format!("...{}", hint_chars.chars().rev().collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_hint() {
        assert_eq!(generate_token_hint("abcdefghijklmnop"), "...mnop");
        assert_eq!(generate_token_hint("abc"), "...abc");
        assert_eq!(generate_token_hint("ab"), "...ab");
    }

    #[test]
    fn test_no_token_status() {
        let status = ClaudeOAuthTokenStatus::no_token();
        assert!(!status.has_token);
        assert!(status.token_hint.is_none());
        assert!(!status.is_expired);
    }

    #[test]
    fn test_obfuscate_token() {
        // Long token should show first 8 and last 4 chars
        let long_token = "v1:abcdefghijklmnopqrstuvwxyz";
        let obfuscated = obfuscate_token(long_token);
        assert_eq!(obfuscated, "v1:abcde...wxyz");
        assert!(!obfuscated.contains("jklmnopqrstuv"));

        // Short token should be fully redacted
        let short_token = "short";
        let obfuscated = obfuscate_token(short_token);
        assert_eq!(obfuscated, "[REDACTED]");

        // Edge case: exactly 12 chars should be redacted
        let edge_token = "123456789012";
        let obfuscated = obfuscate_token(edge_token);
        assert_eq!(obfuscated, "[REDACTED]");

        // 13 chars should show prefix and suffix
        let token_13 = "1234567890123";
        let obfuscated = obfuscate_token(token_13);
        assert_eq!(obfuscated, "12345678...0123");
    }

    #[test]
    fn test_debug_obfuscates_token() {
        let token = ClaudeOAuthToken {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            encrypted_token: "v1:very-secret-token-value-that-should-not-appear-in-logs"
                .to_string(),
            token_hint: Some("...logs".to_string()),
            created_at: Utc::now(),
            expires_at: None,
            last_used_at: None,
        };

        let debug_output = format!("{:?}", token);

        // Should contain obfuscated version
        assert!(debug_output.contains("v1:very-...logs"));

        // Should NOT contain the middle part of the token
        assert!(!debug_output.contains("secret-token-value-that-should-not-appear-in"));
    }
}
