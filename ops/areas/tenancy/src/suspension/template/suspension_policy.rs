use super::*;

use acme_kit::text;

/// Where a suspension policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionPolicyStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SuspensionPolicyStatus {
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

impl std::fmt::Display for SuspensionPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionPolicy {
    pub code: String,
    pub title: String,
    pub source: String,
    pub tenant_id: String,
    pub cost: f64,
    pub status: SuspensionPolicyStatus,
}

impl SuspensionPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionPolicyStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionPolicyStatus) -> bool {
        match self.status {
            SuspensionPolicyStatus::Open => {
                match next {
                    SuspensionPolicyStatus::InReview | SuspensionPolicyStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionPolicyStatus::InReview => {
                return next == SuspensionPolicyStatus::Reopened || next == SuspensionPolicyStatus::Archived;
            }
            SuspensionPolicyStatus::Resolved => {
                return next == SuspensionPolicyStatus::Reopened;
            }
            SuspensionPolicyStatus::Reopened => {
                return next == SuspensionPolicyStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> SuspensionPolicy {
        SuspensionPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            tenant_id: self.tenant_id.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a suspension policy in status open.
pub fn new_suspension_policy(code: &str, title: &str) -> SuspensionPolicy {
    SuspensionPolicy {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        tenant_id: "".to_string(),
        cost: 0.0,
        status: SuspensionPolicyStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_suspension_policy("code-971", "title-950");
        assert_eq!(item.status, SuspensionPolicyStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_suspension_policy("code-971", "title-950");
        if item.move_to(SuspensionPolicyStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionPolicyStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_suspension_policy("code-971", "title-950");
        item.status = SuspensionPolicyStatus::Archived;
        assert!(
            item.move_to(SuspensionPolicyStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
