use super::*;

use acme_kit::text;

/// Where a rolling egress schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingEgressScheduleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingEgressScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingEgressScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling egress schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressSchedule {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub contact_email: String,
    pub port: i64,
    pub status: RollingEgressScheduleStatus,
}

impl RollingEgressSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingEgressScheduleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingEgressScheduleStatus) -> bool {
        match self.status {
            RollingEgressScheduleStatus::Proposed => {
                match next {
                    RollingEgressScheduleStatus::Rejected | RollingEgressScheduleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingEgressScheduleStatus::Approved => {
                match next {
                    RollingEgressScheduleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingEgressScheduleStatus::Rejected => {
                return next == RollingEgressScheduleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingEgressScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling egress schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> RollingEgressSchedule {
        RollingEgressSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            contact_email: self.contact_email.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling egress schedule in status proposed.
pub fn new_rolling_egress_schedule(code: &str, label: &str) -> RollingEgressSchedule {
    RollingEgressSchedule {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        contact_email: "".to_string(),
        port: 0,
        status: RollingEgressScheduleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_egress_schedule("code-848", "label-931");
        assert_eq!(item.status, RollingEgressScheduleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_rolling_egress_schedule("code-848", "label-931");
        if item.move_to(RollingEgressScheduleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingEgressScheduleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_egress_schedule("code-848", "label-931");
        item.status = RollingEgressScheduleStatus::Withdrawn;
        assert!(
            item.move_to(RollingEgressScheduleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
