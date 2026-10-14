use super::*;

use acme_kit::text;

/// Where a legacy usage assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyUsageAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyUsageAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for LegacyUsageAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy usage assignment as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageAssignment {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub threshold: f64,
    pub score: f64,
    pub currency: String,
    pub scope: String,
    pub status: LegacyUsageAssignmentStatus,
}

impl LegacyUsageAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyUsageAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyUsageAssignmentStatus) -> bool {
        match self.status {
            LegacyUsageAssignmentStatus::Draft => {
                return next == LegacyUsageAssignmentStatus::Active;
            }
            LegacyUsageAssignmentStatus::Pending => {
                match next {
                    LegacyUsageAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyUsageAssignmentStatus::Active => {
                return next == LegacyUsageAssignmentStatus::Suspended;
            }
            LegacyUsageAssignmentStatus::Suspended => {
                return next == LegacyUsageAssignmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyUsageAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy usage assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.score * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyUsageAssignment {
        LegacyUsageAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            threshold: self.threshold,
            score: self.score,
            currency: self.currency.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a legacy usage assignment in status draft.
pub fn new_legacy_usage_assignment(code: &str, name: &str) -> LegacyUsageAssignment {
    LegacyUsageAssignment {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        threshold: 0.0,
        score: 0.0,
        currency: "".to_string(),
        scope: "".to_string(),
        status: LegacyUsageAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_usage_assignment("code-72", "name-663");
        assert_eq!(item.status, LegacyUsageAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_legacy_usage_assignment("code-72", "name-663");
        if item.move_to(LegacyUsageAssignmentStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyUsageAssignmentStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_usage_assignment("code-72", "name-663");
        item.status = LegacyUsageAssignmentStatus::Closed;
        assert!(
            item.move_to(LegacyUsageAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
