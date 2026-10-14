use super::*;

use acme_kit::text;

/// Where a adaptive canary assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveCanaryAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveCanaryAssignmentStatus {
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

impl std::fmt::Display for AdaptiveCanaryAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive canary assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryAssignment {
    pub code: String,
    pub label: String,
    pub country: String,
    pub checksum: String,
    pub key_id: String,
    pub channel: String,
    pub visible: bool,
    pub currency: String,
    pub score: f64,
    pub status: AdaptiveCanaryAssignmentStatus,
}

impl AdaptiveCanaryAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveCanaryAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveCanaryAssignmentStatus) -> bool {
        match self.status {
            AdaptiveCanaryAssignmentStatus::Open => {
                match next {
                    AdaptiveCanaryAssignmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveCanaryAssignmentStatus::InReview => {
                match next {
                    AdaptiveCanaryAssignmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveCanaryAssignmentStatus::Resolved => {
                return next == AdaptiveCanaryAssignmentStatus::Reopened
                    || next == AdaptiveCanaryAssignmentStatus::Archived;
            }
            AdaptiveCanaryAssignmentStatus::Reopened => {
                return next == AdaptiveCanaryAssignmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveCanaryAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive canary assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> AdaptiveCanaryAssignment {
        AdaptiveCanaryAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            checksum: self.checksum.clone(),
            key_id: self.key_id.clone(),
            channel: self.channel.clone(),
            visible: self.visible,
            currency: self.currency.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a adaptive canary assignment in status open.
pub fn new_adaptive_canary_assignment(code: &str, label: &str) -> AdaptiveCanaryAssignment {
    AdaptiveCanaryAssignment {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        checksum: "".to_string(),
        key_id: "".to_string(),
        channel: "".to_string(),
        visible: false,
        currency: "".to_string(),
        score: 0.0,
        status: AdaptiveCanaryAssignmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_canary_assignment("code-574", "label-170");
        assert_eq!(item.status, AdaptiveCanaryAssignmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_adaptive_canary_assignment("code-574", "label-170");
        if item.move_to(AdaptiveCanaryAssignmentStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveCanaryAssignmentStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_canary_assignment("code-574", "label-170");
        item.status = AdaptiveCanaryAssignmentStatus::Archived;
        assert!(
            item.move_to(AdaptiveCanaryAssignmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
