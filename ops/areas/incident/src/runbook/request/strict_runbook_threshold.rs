use super::*;

use acme_kit::text;

/// Where a strict runbook threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRunbookThresholdStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictRunbookThresholdStatus {
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

impl std::fmt::Display for StrictRunbookThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict runbook threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRunbookThreshold {
    pub code: String,
    pub title: String,
    pub attempts: i64,
    pub label: String,
    pub floor_value: f64,
    pub key_id: String,
    pub description: String,
    pub audience: String,
    pub margin: f64,
    pub approved: bool,
    pub status: StrictRunbookThresholdStatus,
}

impl StrictRunbookThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRunbookThresholdStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRunbookThresholdStatus) -> bool {
        match self.status {
            StrictRunbookThresholdStatus::Open => {
                return next == StrictRunbookThresholdStatus::Reopened;
            }
            StrictRunbookThresholdStatus::InReview => {
                return next == StrictRunbookThresholdStatus::Resolved;
            }
            StrictRunbookThresholdStatus::Resolved => {
                return next == StrictRunbookThresholdStatus::Reopened
                    || next == StrictRunbookThresholdStatus::Archived;
            }
            StrictRunbookThresholdStatus::Reopened => {
                match next {
                    StrictRunbookThresholdStatus::Archived => {
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

    pub fn move_to(&mut self, next: StrictRunbookThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict runbook threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.attempts as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictRunbookThreshold {
        StrictRunbookThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            label: self.label.clone(),
            floor_value: self.floor_value,
            key_id: self.key_id.clone(),
            description: self.description.clone(),
            audience: self.audience.clone(),
            margin: self.margin,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict runbook threshold in status open.
pub fn new_strict_runbook_threshold(code: &str, title: &str) -> StrictRunbookThreshold {
    StrictRunbookThreshold {
        code: code.to_string(),
        title: title.to_string(),
        attempts: 0,
        label: "".to_string(),
        floor_value: 0.0,
        key_id: "".to_string(),
        description: "".to_string(),
        audience: "".to_string(),
        margin: 0.0,
        approved: false,
        status: StrictRunbookThresholdStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_runbook_threshold("code-838", "title-574");
        assert_eq!(item.status, StrictRunbookThresholdStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_strict_runbook_threshold("code-838", "title-574");
        if item.move_to(StrictRunbookThresholdStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictRunbookThresholdStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_runbook_threshold("code-838", "title-574");
        item.status = StrictRunbookThresholdStatus::Archived;
        assert!(
            item.move_to(StrictRunbookThresholdStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
