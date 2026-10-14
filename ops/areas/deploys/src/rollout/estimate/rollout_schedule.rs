use super::*;

use acme_kit::text;

/// Where a rollout schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RolloutScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RolloutScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout schedule as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSchedule {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub contact_email: String,
    pub ceiling: f64,
    pub quota_units: i64,
    pub segment: String,
    pub ends_at: i64,
    pub quantity: i64,
    pub reviewer: String,
    pub status: RolloutScheduleStatus,
}

impl RolloutSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutScheduleStatus) -> bool {
        match self.status {
            RolloutScheduleStatus::Requested => {
                return next == RolloutScheduleStatus::Granted;
            }
            RolloutScheduleStatus::Granted => {
                match next {
                    RolloutScheduleStatus::Revoked | RolloutScheduleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutScheduleStatus::Revoked => {
                match next {
                    RolloutScheduleStatus::Expired => {
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

    pub fn move_to(&mut self, next: RolloutScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ceiling + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutSchedule {
        RolloutSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            contact_email: self.contact_email.clone(),
            ceiling: self.ceiling,
            quota_units: self.quota_units,
            segment: self.segment.clone(),
            ends_at: self.ends_at,
            quantity: self.quantity,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rollout schedule in status requested.
pub fn new_rollout_schedule(code: &str, label: &str) -> RolloutSchedule {
    RolloutSchedule {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        contact_email: "".to_string(),
        ceiling: 0.0,
        quota_units: 0,
        segment: "".to_string(),
        ends_at: 0,
        quantity: 0,
        reviewer: "".to_string(),
        status: RolloutScheduleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rollout_schedule("code-122", "label-615");
        assert_eq!(item.status, RolloutScheduleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_rollout_schedule("code-122", "label-615");
        if item.move_to(RolloutScheduleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutScheduleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rollout_schedule("code-122", "label-615");
        item.status = RolloutScheduleStatus::Expired;
        assert!(
            item.move_to(RolloutScheduleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
