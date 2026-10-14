use super::*;

use acme_kit::text;

/// Where a strict health check target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictHealthCheckTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictHealthCheckTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for StrictHealthCheckTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict health check target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckTarget {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub discount: f64,
    pub archived: bool,
    pub source: String,
    pub port: i64,
    pub key_id: String,
    pub ratio: f64,
    pub expires_at: i64,
    pub scope: String,
    pub status: StrictHealthCheckTargetStatus,
}

impl StrictHealthCheckTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictHealthCheckTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictHealthCheckTargetStatus) -> bool {
        match self.status {
            StrictHealthCheckTargetStatus::Open => {
                match next {
                    StrictHealthCheckTargetStatus::InReview | StrictHealthCheckTargetStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictHealthCheckTargetStatus::InReview => {
                match next {
                    StrictHealthCheckTargetStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictHealthCheckTargetStatus::Resolved => {
                match next {
                    StrictHealthCheckTargetStatus::Reopened | StrictHealthCheckTargetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictHealthCheckTargetStatus::Reopened => {
                match next {
                    StrictHealthCheckTargetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictHealthCheckTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict health check target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.port as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictHealthCheckTarget {
        StrictHealthCheckTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            discount: self.discount,
            archived: self.archived,
            source: self.source.clone(),
            port: self.port,
            key_id: self.key_id.clone(),
            ratio: self.ratio,
            expires_at: self.expires_at,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a strict health check target in status open.
pub fn new_strict_health_check_target(code: &str, label: &str) -> StrictHealthCheckTarget {
    StrictHealthCheckTarget {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        discount: 0.0,
        archived: false,
        source: "".to_string(),
        port: 0,
        key_id: "".to_string(),
        ratio: 0.0,
        expires_at: 0,
        scope: "".to_string(),
        status: StrictHealthCheckTargetStatus::Open,
    }
}
