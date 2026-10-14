use super::*;

use acme_kit::text;

/// Where a strict nonce schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictNonceScheduleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl StrictNonceScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for StrictNonceScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict nonce schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictNonceSchedule {
    pub code: String,
    pub title: String,
    pub country: String,
    pub budget: f64,
    pub visible: bool,
    pub cluster: String,
    pub pinned: bool,
    pub quantity: i64,
    pub segment: String,
    pub status: StrictNonceScheduleStatus,
}

impl StrictNonceSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictNonceScheduleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictNonceScheduleStatus) -> bool {
        match self.status {
            StrictNonceScheduleStatus::Proposed => {
                return next == StrictNonceScheduleStatus::Withdrawn;
            }
            StrictNonceScheduleStatus::Approved => {
                match next {
                    StrictNonceScheduleStatus::Rejected | StrictNonceScheduleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictNonceScheduleStatus::Rejected => {
                match next {
                    StrictNonceScheduleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: StrictNonceScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict nonce schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.budget - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictNonceSchedule {
        StrictNonceSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            budget: self.budget,
            visible: self.visible,
            cluster: self.cluster.clone(),
            pinned: self.pinned,
            quantity: self.quantity,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a strict nonce schedule in status proposed.
pub fn new_strict_nonce_schedule(code: &str, title: &str) -> StrictNonceSchedule {
    StrictNonceSchedule {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        budget: 0.0,
        visible: false,
        cluster: "".to_string(),
        pinned: false,
        quantity: 0,
        segment: "".to_string(),
        status: StrictNonceScheduleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_strict_nonce_schedule("code-181", "title-968");
        assert_eq!(item.status, StrictNonceScheduleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_strict_nonce_schedule("code-181", "title-968");
        if item.move_to(StrictNonceScheduleStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictNonceScheduleStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_strict_nonce_schedule("code-181", "title-968");
        item.status = StrictNonceScheduleStatus::Withdrawn;
        assert!(
            item.move_to(StrictNonceScheduleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
