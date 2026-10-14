use super::*;

use acme_kit::text;

/// Where a regional timeline request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalTimelineRequestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalTimelineRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalTimelineRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional timeline request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineRequest {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub quantity: i64,
    pub revision: i64,
    pub approved: bool,
    pub amount: f64,
    pub status: RegionalTimelineRequestStatus,
}

impl RegionalTimelineRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalTimelineRequestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalTimelineRequestStatus) -> bool {
        match self.status {
            RegionalTimelineRequestStatus::Proposed => {
                return next == RegionalTimelineRequestStatus::Approved
                    || next == RegionalTimelineRequestStatus::Rejected;
            }
            RegionalTimelineRequestStatus::Approved => {
                return next == RegionalTimelineRequestStatus::Rejected
                    || next == RegionalTimelineRequestStatus::Withdrawn;
            }
            RegionalTimelineRequestStatus::Rejected => {
                return next == RegionalTimelineRequestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalTimelineRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional timeline request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional timeline request", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.quantity as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalTimelineRequest {
        RegionalTimelineRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            quantity: self.quantity,
            revision: self.revision,
            approved: self.approved,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional timeline request in status proposed.
pub fn new_regional_timeline_request(code: &str, title: &str) -> RegionalTimelineRequest {
    RegionalTimelineRequest {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        quantity: 0,
        revision: 0,
        approved: false,
        amount: 0.0,
        status: RegionalTimelineRequestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_timeline_request("code-747", "title-694");
        assert_eq!(item.status, RegionalTimelineRequestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_regional_timeline_request("code-747", "title-694");
        if item.move_to(RegionalTimelineRequestStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalTimelineRequestStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_timeline_request("code-747", "title-694");
        item.status = RegionalTimelineRequestStatus::Withdrawn;
        assert!(
            item.move_to(RegionalTimelineRequestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
