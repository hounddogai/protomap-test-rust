use super::*;

use acme_kit::text;

/// Where a canary threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryThresholdStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CanaryThresholdStatus {
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

impl std::fmt::Display for CanaryThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a canary threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryThreshold {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub weight: f64,
    pub burst: i64,
    pub status: CanaryThresholdStatus,
}

impl CanaryThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryThresholdStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryThresholdStatus) -> bool {
        match self.status {
            CanaryThresholdStatus::Open => {
                match next {
                    CanaryThresholdStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryThresholdStatus::InReview => {
                return next == CanaryThresholdStatus::Archived;
            }
            CanaryThresholdStatus::Resolved => {
                return next == CanaryThresholdStatus::Reopened || next == CanaryThresholdStatus::Archived;
            }
            CanaryThresholdStatus::Reopened => {
                return next == CanaryThresholdStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.burst as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryThreshold {
        CanaryThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            weight: self.weight,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a canary threshold in status open.
pub fn new_canary_threshold(code: &str, title: &str) -> CanaryThreshold {
    CanaryThreshold {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        weight: 0.0,
        burst: 0,
        status: CanaryThresholdStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_canary_threshold("code-529", "title-637");
        assert_eq!(item.status, CanaryThresholdStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_canary_threshold("code-529", "title-637");
        if item.move_to(CanaryThresholdStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryThresholdStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_canary_threshold("code-529", "title-637");
        item.status = CanaryThresholdStatus::Archived;
        assert!(
            item.move_to(CanaryThresholdStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
