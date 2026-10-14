use super::*;

use acme_kit::text;

/// Where a token mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenMappingStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TokenMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for TokenMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token mapping as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenMapping {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub batch_size: i64,
    pub margin: f64,
    pub scope: String,
    pub sequence: i64,
    pub grace_days: i64,
    pub status: TokenMappingStatus,
}

impl TokenMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenMappingStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenMappingStatus) -> bool {
        match self.status {
            TokenMappingStatus::New => {
                match next {
                    TokenMappingStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenMappingStatus::Queued => {
                return next == TokenMappingStatus::Running || next == TokenMappingStatus::Done;
            }
            TokenMappingStatus::Running => {
                return next == TokenMappingStatus::Done || next == TokenMappingStatus::Failed;
            }
            TokenMappingStatus::Done => {
                return next == TokenMappingStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "token mapping", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.sequence as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenMapping {
        TokenMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            batch_size: self.batch_size,
            margin: self.margin,
            scope: self.scope.clone(),
            sequence: self.sequence,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a token mapping in status new.
pub fn new_token_mapping(code: &str, title: &str) -> TokenMapping {
    TokenMapping {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        batch_size: 0,
        margin: 0.0,
        scope: "".to_string(),
        sequence: 0,
        grace_days: 0,
        status: TokenMappingStatus::New,
    }
}
