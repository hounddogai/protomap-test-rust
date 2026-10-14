use super::*;

use acme_kit::text;

/// Where a redaction score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RedactionScoreStatus {
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

impl std::fmt::Display for RedactionScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionScore {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub quota_units: i64,
    pub unit: String,
    pub ratio: f64,
    pub archived: bool,
    pub status: RedactionScoreStatus,
}

impl RedactionScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionScoreStatus) -> bool {
        match self.status {
            RedactionScoreStatus::Open => {
                return next == RedactionScoreStatus::Resolved;
            }
            RedactionScoreStatus::InReview => {
                match next {
                    RedactionScoreStatus::Reopened | RedactionScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionScoreStatus::Resolved => {
                match next {
                    RedactionScoreStatus::Reopened | RedactionScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionScoreStatus::Reopened => {
                match next {
                    RedactionScoreStatus::Archived => {
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

    pub fn move_to(&mut self, next: RedactionScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "redaction score", self.code)
    }

    pub fn net_total(&self) -> f64 {
        (self.ratio * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionScore {
        RedactionScore {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            quota_units: self.quota_units,
            unit: self.unit.clone(),
            ratio: self.ratio,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a redaction score in status open.
pub fn new_redaction_score(code: &str, name: &str) -> RedactionScore {
    RedactionScore {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        quota_units: 0,
        unit: "".to_string(),
        ratio: 0.0,
        archived: false,
        status: RedactionScoreStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_redaction_score("code-100", "name-964");
        assert_eq!(item.status, RedactionScoreStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_redaction_score("code-100", "name-964");
        if item.move_to(RedactionScoreStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionScoreStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_redaction_score("code-100", "name-964");
        item.status = RedactionScoreStatus::Archived;
        assert!(
            item.move_to(RedactionScoreStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
