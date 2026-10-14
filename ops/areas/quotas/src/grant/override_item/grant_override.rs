use super::*;

use acme_kit::text;

/// Where a grant override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GrantOverrideStatus {
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

impl std::fmt::Display for GrantOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One grant override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantOverride {
    pub code: String,
    pub label: String,
    pub region: String,
    pub capacity: i64,
    pub minimum: f64,
    pub sequence: i64,
    pub budget: f64,
    pub status: GrantOverrideStatus,
}

impl GrantOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantOverrideStatus) -> bool {
        match self.status {
            GrantOverrideStatus::Open => {
                match next {
                    GrantOverrideStatus::Resolved | GrantOverrideStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantOverrideStatus::InReview => {
                return next == GrantOverrideStatus::Archived;
            }
            GrantOverrideStatus::Resolved => {
                return next == GrantOverrideStatus::Reopened || next == GrantOverrideStatus::Archived;
            }
            GrantOverrideStatus::Reopened => {
                match next {
                    GrantOverrideStatus::Archived => {
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

    pub fn move_to(&mut self, next: GrantOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.capacity as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantOverride {
        GrantOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            capacity: self.capacity,
            minimum: self.minimum,
            sequence: self.sequence,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a grant override in status open.
pub fn new_grant_override(code: &str, label: &str) -> GrantOverride {
    GrantOverride {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        capacity: 0,
        minimum: 0.0,
        sequence: 0,
        budget: 0.0,
        status: GrantOverrideStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_grant_override("code-961", "label-612");
        assert_eq!(item.status, GrantOverrideStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_grant_override("code-961", "label-612");
        if item.move_to(GrantOverrideStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantOverrideStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_grant_override("code-961", "label-612");
        item.status = GrantOverrideStatus::Archived;
        assert!(
            item.move_to(GrantOverrideStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
