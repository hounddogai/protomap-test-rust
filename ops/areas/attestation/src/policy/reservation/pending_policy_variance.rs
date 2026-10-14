use super::*;

use acme_kit::text;

/// Where a pending policy variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPolicyVarianceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingPolicyVarianceStatus {
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

impl std::fmt::Display for PendingPolicyVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending policy variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyVariance {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub ratio: f64,
    pub pinned: bool,
    pub limit: i64,
    pub status: PendingPolicyVarianceStatus,
}

impl PendingPolicyVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPolicyVarianceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPolicyVarianceStatus) -> bool {
        match self.status {
            PendingPolicyVarianceStatus::Open => {
                return next == PendingPolicyVarianceStatus::Reopened || next == PendingPolicyVarianceStatus::Archived;
            }
            PendingPolicyVarianceStatus::InReview => {
                match next {
                    PendingPolicyVarianceStatus::Resolved | PendingPolicyVarianceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPolicyVarianceStatus::Resolved => {
                return next == PendingPolicyVarianceStatus::Reopened;
            }
            PendingPolicyVarianceStatus::Reopened => {
                match next {
                    PendingPolicyVarianceStatus::Archived => {
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

    pub fn move_to(&mut self, next: PendingPolicyVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending policy variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.ratio + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPolicyVariance {
        PendingPolicyVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            ratio: self.ratio,
            pinned: self.pinned,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending policy variance in status open.
pub fn new_pending_policy_variance(code: &str, title: &str) -> PendingPolicyVariance {
    PendingPolicyVariance {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        ratio: 0.0,
        pinned: false,
        limit: 0,
        status: PendingPolicyVarianceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_policy_variance("code-505", "title-430");
        assert_eq!(item.status, PendingPolicyVarianceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_pending_policy_variance("code-505", "title-430");
        if item.move_to(PendingPolicyVarianceStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPolicyVarianceStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_policy_variance("code-505", "title-430");
        item.status = PendingPolicyVarianceStatus::Archived;
        assert!(
            item.move_to(PendingPolicyVarianceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
