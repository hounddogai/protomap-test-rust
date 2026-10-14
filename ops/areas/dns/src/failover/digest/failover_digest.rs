use super::*;

use acme_kit::text;

/// Where a failover digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FailoverDigestStatus {
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

impl std::fmt::Display for FailoverDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One failover digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverDigest {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub burst: i64,
    pub host: String,
    pub updated_at: i64,
    pub margin: f64,
    pub name: String,
    pub namespace: String,
    pub status: FailoverDigestStatus,
}

impl FailoverDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverDigestStatus) -> bool {
        match self.status {
            FailoverDigestStatus::Open => {
                return next == FailoverDigestStatus::InReview;
            }
            FailoverDigestStatus::InReview => {
                match next {
                    FailoverDigestStatus::Reopened | FailoverDigestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverDigestStatus::Resolved => {
                match next {
                    FailoverDigestStatus::Reopened | FailoverDigestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverDigestStatus::Reopened => {
                match next {
                    FailoverDigestStatus::Archived => {
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

    pub fn move_to(&mut self, next: FailoverDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.updated_at as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverDigest {
        FailoverDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            burst: self.burst,
            host: self.host.clone(),
            updated_at: self.updated_at,
            margin: self.margin,
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a failover digest in status open.
pub fn new_failover_digest(code: &str, title: &str) -> FailoverDigest {
    FailoverDigest {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        burst: 0,
        host: "".to_string(),
        updated_at: 0,
        margin: 0.0,
        name: "".to_string(),
        namespace: "".to_string(),
        status: FailoverDigestStatus::Open,
    }
}
