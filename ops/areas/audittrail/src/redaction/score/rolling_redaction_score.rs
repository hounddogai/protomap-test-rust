use super::*;

use acme_kit::text;

/// Where a rolling redaction score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRedactionScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingRedactionScoreStatus {
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

impl std::fmt::Display for RollingRedactionScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling redaction score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionScore {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub reviewer: String,
    pub host: String,
    pub batch_size: i64,
    pub starts_at: i64,
    pub namespace: String,
    pub tenant_id: String,
    pub margin: f64,
    pub pinned: bool,
    pub status: RollingRedactionScoreStatus,
}

impl RollingRedactionScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRedactionScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRedactionScoreStatus) -> bool {
        match self.status {
            RollingRedactionScoreStatus::Open => {
                match next {
                    RollingRedactionScoreStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRedactionScoreStatus::InReview => {
                match next {
                    RollingRedactionScoreStatus::Resolved | RollingRedactionScoreStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRedactionScoreStatus::Resolved => {
                return next == RollingRedactionScoreStatus::Archived;
            }
            RollingRedactionScoreStatus::Reopened => {
                match next {
                    RollingRedactionScoreStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollingRedactionScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling redaction score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling redaction score", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.starts_at as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRedactionScore {
        RollingRedactionScore {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            reviewer: self.reviewer.clone(),
            host: self.host.clone(),
            batch_size: self.batch_size,
            starts_at: self.starts_at,
            namespace: self.namespace.clone(),
            tenant_id: self.tenant_id.clone(),
            margin: self.margin,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rolling redaction score in status open.
pub fn new_rolling_redaction_score(code: &str, name: &str) -> RollingRedactionScore {
    RollingRedactionScore {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        reviewer: "".to_string(),
        host: "".to_string(),
        batch_size: 0,
        starts_at: 0,
        namespace: "".to_string(),
        tenant_id: "".to_string(),
        margin: 0.0,
        pinned: false,
        status: RollingRedactionScoreStatus::Open,
    }
}
