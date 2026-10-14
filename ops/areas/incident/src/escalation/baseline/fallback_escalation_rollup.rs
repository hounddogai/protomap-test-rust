use super::*;

use acme_kit::text;

/// Where a fallback escalation rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackEscalationRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackEscalationRollupStatus {
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

impl std::fmt::Display for FallbackEscalationRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback escalation rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEscalationRollup {
    pub code: String,
    pub title: String,
    pub label: String,
    pub country: String,
    pub name: String,
    pub notes: String,
    pub score: f64,
    pub status: FallbackEscalationRollupStatus,
}

impl FallbackEscalationRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackEscalationRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackEscalationRollupStatus) -> bool {
        match self.status {
            FallbackEscalationRollupStatus::Open => {
                match next {
                    FallbackEscalationRollupStatus::InReview | FallbackEscalationRollupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEscalationRollupStatus::InReview => {
                match next {
                    FallbackEscalationRollupStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEscalationRollupStatus::Resolved => {
                return next == FallbackEscalationRollupStatus::Archived;
            }
            FallbackEscalationRollupStatus::Reopened => {
                return next == FallbackEscalationRollupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackEscalationRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback escalation rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback escalation rollup", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> FallbackEscalationRollup {
        FallbackEscalationRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a fallback escalation rollup in status open.
pub fn new_fallback_escalation_rollup(code: &str, title: &str) -> FallbackEscalationRollup {
    FallbackEscalationRollup {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        country: "".to_string(),
        name: "".to_string(),
        notes: "".to_string(),
        score: 0.0,
        status: FallbackEscalationRollupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_escalation_rollup("code-986", "title-548");
        assert_eq!(item.status, FallbackEscalationRollupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_fallback_escalation_rollup("code-986", "title-548");
        if item.move_to(FallbackEscalationRollupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackEscalationRollupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_escalation_rollup("code-986", "title-548");
        item.status = FallbackEscalationRollupStatus::Archived;
        assert!(
            item.move_to(FallbackEscalationRollupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
