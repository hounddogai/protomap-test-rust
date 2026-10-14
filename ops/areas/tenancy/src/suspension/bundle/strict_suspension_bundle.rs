use super::*;

use acme_kit::text;

/// Where a strict suspension bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictSuspensionBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictSuspensionBundleStatus {
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

impl std::fmt::Display for StrictSuspensionBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict suspension bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionBundle {
    pub code: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub key_id: String,
    pub score: f64,
    pub status: StrictSuspensionBundleStatus,
}

impl StrictSuspensionBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictSuspensionBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictSuspensionBundleStatus) -> bool {
        match self.status {
            StrictSuspensionBundleStatus::Open => {
                return next == StrictSuspensionBundleStatus::Reopened;
            }
            StrictSuspensionBundleStatus::InReview => {
                return next == StrictSuspensionBundleStatus::Reopened
                    || next == StrictSuspensionBundleStatus::Archived;
            }
            StrictSuspensionBundleStatus::Resolved => {
                return next == StrictSuspensionBundleStatus::Archived;
            }
            StrictSuspensionBundleStatus::Reopened => {
                return next == StrictSuspensionBundleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictSuspensionBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict suspension bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict suspension bundle", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> StrictSuspensionBundle {
        StrictSuspensionBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            category: self.category.clone(),
            key_id: self.key_id.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict suspension bundle in status open.
pub fn new_strict_suspension_bundle(code: &str, name: &str) -> StrictSuspensionBundle {
    StrictSuspensionBundle {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        category: "".to_string(),
        key_id: "".to_string(),
        score: 0.0,
        status: StrictSuspensionBundleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_suspension_bundle("code-968", "name-868");
        assert_eq!(item.status, StrictSuspensionBundleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_strict_suspension_bundle("code-968", "name-868");
        if item.move_to(StrictSuspensionBundleStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictSuspensionBundleStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_suspension_bundle("code-968", "name-868");
        item.status = StrictSuspensionBundleStatus::Archived;
        assert!(
            item.move_to(StrictSuspensionBundleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
