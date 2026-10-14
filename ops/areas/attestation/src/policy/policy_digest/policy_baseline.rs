use super::*;

use acme_kit::text;

/// Where a policy baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyBaselineStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PolicyBaselineStatus {
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

impl std::fmt::Display for PolicyBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a policy baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyBaseline {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub fingerprint: String,
    pub amount: f64,
    pub status: PolicyBaselineStatus,
}

impl PolicyBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyBaselineStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyBaselineStatus) -> bool {
        match self.status {
            PolicyBaselineStatus::Open => {
                return next == PolicyBaselineStatus::Resolved || next == PolicyBaselineStatus::Reopened;
            }
            PolicyBaselineStatus::InReview => {
                match next {
                    PolicyBaselineStatus::Reopened | PolicyBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyBaselineStatus::Resolved => {
                match next {
                    PolicyBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyBaselineStatus::Reopened => {
                return next == PolicyBaselineStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "policy baseline", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> PolicyBaseline {
        PolicyBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            fingerprint: self.fingerprint.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a policy baseline in status open.
pub fn new_policy_baseline(code: &str, title: &str) -> PolicyBaseline {
    PolicyBaseline {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        fingerprint: "".to_string(),
        amount: 0.0,
        status: PolicyBaselineStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_policy_baseline("code-868", "title-161");
        assert_eq!(item.status, PolicyBaselineStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_policy_baseline("code-868", "title-161");
        if item.move_to(PolicyBaselineStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyBaselineStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_policy_baseline("code-868", "title-161");
        item.status = PolicyBaselineStatus::Archived;
        assert!(
            item.move_to(PolicyBaselineStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
