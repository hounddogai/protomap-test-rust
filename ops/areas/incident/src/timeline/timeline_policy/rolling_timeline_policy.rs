use super::*;

use acme_kit::text;

/// Where a rolling timeline policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTimelinePolicyStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingTimelinePolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingTimelinePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling timeline policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelinePolicy {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub summary: String,
    pub quantity: i64,
    pub status: RollingTimelinePolicyStatus,
}

impl RollingTimelinePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTimelinePolicyStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTimelinePolicyStatus) -> bool {
        match self.status {
            RollingTimelinePolicyStatus::Proposed => {
                match next {
                    RollingTimelinePolicyStatus::Approved | RollingTimelinePolicyStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTimelinePolicyStatus::Approved => {
                match next {
                    RollingTimelinePolicyStatus::Rejected | RollingTimelinePolicyStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTimelinePolicyStatus::Rejected => {
                match next {
                    RollingTimelinePolicyStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RollingTimelinePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling timeline policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling timeline policy", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> RollingTimelinePolicy {
        RollingTimelinePolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            summary: self.summary.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling timeline policy in status proposed.
pub fn new_rolling_timeline_policy(code: &str, title: &str) -> RollingTimelinePolicy {
    RollingTimelinePolicy {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        summary: "".to_string(),
        quantity: 0,
        status: RollingTimelinePolicyStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_timeline_policy("code-106", "title-672");
        assert_eq!(item.status, RollingTimelinePolicyStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rolling_timeline_policy("code-106", "title-672");
        if item.move_to(RollingTimelinePolicyStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingTimelinePolicyStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_timeline_policy("code-106", "title-672");
        item.status = RollingTimelinePolicyStatus::Withdrawn;
        assert!(
            item.move_to(RollingTimelinePolicyStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
