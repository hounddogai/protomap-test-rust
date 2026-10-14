use super::*;

use acme_kit::text;

/// Where a identity target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl IdentityTargetStatus {
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

impl std::fmt::Display for IdentityTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity target as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTarget {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub sequence: i64,
    pub phone: String,
    pub locale: String,
    pub attempts: i64,
    pub margin: f64,
    pub minimum: f64,
    pub status: IdentityTargetStatus,
}

impl IdentityTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityTargetStatus) -> bool {
        match self.status {
            IdentityTargetStatus::Open => {
                return next == IdentityTargetStatus::Resolved || next == IdentityTargetStatus::Archived;
            }
            IdentityTargetStatus::InReview => {
                return next == IdentityTargetStatus::Resolved || next == IdentityTargetStatus::Archived;
            }
            IdentityTargetStatus::Resolved => {
                return next == IdentityTargetStatus::Reopened || next == IdentityTargetStatus::Archived;
            }
            IdentityTargetStatus::Reopened => {
                match next {
                    IdentityTargetStatus::Archived => {
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

    pub fn move_to(&mut self, next: IdentityTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.attempts as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityTarget {
        IdentityTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            sequence: self.sequence,
            phone: self.phone.clone(),
            locale: self.locale.clone(),
            attempts: self.attempts,
            margin: self.margin,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a identity target in status open.
pub fn new_identity_target(code: &str, title: &str) -> IdentityTarget {
    IdentityTarget {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        sequence: 0,
        phone: "".to_string(),
        locale: "".to_string(),
        attempts: 0,
        margin: 0.0,
        minimum: 0.0,
        status: IdentityTargetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_identity_target("code-791", "title-329");
        assert_eq!(item.status, IdentityTargetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_identity_target("code-791", "title-329");
        if item.move_to(IdentityTargetStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityTargetStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_identity_target("code-791", "title-329");
        item.status = IdentityTargetStatus::Archived;
        assert!(
            item.move_to(IdentityTargetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
