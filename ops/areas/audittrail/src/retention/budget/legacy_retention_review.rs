use super::*;

use acme_kit::text;

/// Where a legacy retention review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRetentionReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyRetentionReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for LegacyRetentionReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy retention review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionReview {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub segment: String,
    pub contact_email: String,
    pub status: LegacyRetentionReviewStatus,
}

impl LegacyRetentionReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRetentionReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRetentionReviewStatus) -> bool {
        match self.status {
            LegacyRetentionReviewStatus::Idle => {
                match next {
                    LegacyRetentionReviewStatus::Warming | LegacyRetentionReviewStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRetentionReviewStatus::Warming => {
                match next {
                    LegacyRetentionReviewStatus::Ready | LegacyRetentionReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRetentionReviewStatus::Ready => {
                return next == LegacyRetentionReviewStatus::Draining;
            }
            LegacyRetentionReviewStatus::Draining => {
                match next {
                    LegacyRetentionReviewStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LegacyRetentionReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy retention review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.floor_value
    }

    pub fn copy(&self) -> LegacyRetentionReview {
        LegacyRetentionReview {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            segment: self.segment.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a legacy retention review in status idle.
pub fn new_legacy_retention_review(code: &str, name: &str) -> LegacyRetentionReview {
    LegacyRetentionReview {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        segment: "".to_string(),
        contact_email: "".to_string(),
        status: LegacyRetentionReviewStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_legacy_retention_review("code-900", "name-765");
        assert_eq!(item.status, LegacyRetentionReviewStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_legacy_retention_review("code-900", "name-765");
        if item.move_to(LegacyRetentionReviewStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRetentionReviewStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_legacy_retention_review("code-900", "name-765");
        item.status = LegacyRetentionReviewStatus::Stopped;
        assert!(
            item.move_to(LegacyRetentionReviewStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
