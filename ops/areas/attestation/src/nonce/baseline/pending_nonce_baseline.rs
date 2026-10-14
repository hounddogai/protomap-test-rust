use super::*;

use acme_kit::text;

/// Where a pending nonce baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingNonceBaselineStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingNonceBaselineStatus {
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

impl std::fmt::Display for PendingNonceBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending nonce baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceBaseline {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub source: String,
    pub cluster: String,
    pub ratio: f64,
    pub ends_at: i64,
    pub status: PendingNonceBaselineStatus,
}

impl PendingNonceBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingNonceBaselineStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingNonceBaselineStatus) -> bool {
        match self.status {
            PendingNonceBaselineStatus::Open => {
                match next {
                    PendingNonceBaselineStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingNonceBaselineStatus::InReview => {
                match next {
                    PendingNonceBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingNonceBaselineStatus::Resolved => {
                return next == PendingNonceBaselineStatus::Archived;
            }
            PendingNonceBaselineStatus::Reopened => {
                match next {
                    PendingNonceBaselineStatus::Archived => {
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

    pub fn move_to(&mut self, next: PendingNonceBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending nonce baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ratio + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingNonceBaseline {
        PendingNonceBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            source: self.source.clone(),
            cluster: self.cluster.clone(),
            ratio: self.ratio,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending nonce baseline in status open.
pub fn new_pending_nonce_baseline(code: &str, name: &str) -> PendingNonceBaseline {
    PendingNonceBaseline {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        source: "".to_string(),
        cluster: "".to_string(),
        ratio: 0.0,
        ends_at: 0,
        status: PendingNonceBaselineStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_nonce_baseline("code-313", "name-944");
        assert_eq!(item.status, PendingNonceBaselineStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_pending_nonce_baseline("code-313", "name-944");
        if item.move_to(PendingNonceBaselineStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingNonceBaselineStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_nonce_baseline("code-313", "name-944");
        item.status = PendingNonceBaselineStatus::Archived;
        assert!(
            item.move_to(PendingNonceBaselineStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
