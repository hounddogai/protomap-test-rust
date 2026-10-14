use super::*;

use acme_kit::text;

/// Where a grant preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantPreferenceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GrantPreferenceStatus {
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

impl std::fmt::Display for GrantPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a grant preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantPreference {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub description: String,
    pub approved: bool,
    pub notes: String,
    pub amount: f64,
    pub status: GrantPreferenceStatus,
}

impl GrantPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantPreferenceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantPreferenceStatus) -> bool {
        match self.status {
            GrantPreferenceStatus::Open => {
                match next {
                    GrantPreferenceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantPreferenceStatus::InReview => {
                match next {
                    GrantPreferenceStatus::Reopened | GrantPreferenceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantPreferenceStatus::Resolved => {
                return next == GrantPreferenceStatus::Archived;
            }
            GrantPreferenceStatus::Reopened => {
                return next == GrantPreferenceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "grant preference", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> GrantPreference {
        GrantPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            description: self.description.clone(),
            approved: self.approved,
            notes: self.notes.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a grant preference in status open.
pub fn new_grant_preference(code: &str, name: &str) -> GrantPreference {
    GrantPreference {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        description: "".to_string(),
        approved: false,
        notes: "".to_string(),
        amount: 0.0,
        status: GrantPreferenceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_grant_preference("code-426", "name-730");
        assert_eq!(item.status, GrantPreferenceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_grant_preference("code-426", "name-730");
        if item.move_to(GrantPreferenceStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantPreferenceStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_grant_preference("code-426", "name-730");
        item.status = GrantPreferenceStatus::Archived;
        assert!(
            item.move_to(GrantPreferenceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
