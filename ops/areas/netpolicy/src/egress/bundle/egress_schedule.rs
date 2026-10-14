use super::*;

use acme_kit::text;

/// Where a egress schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl EgressScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for EgressScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a egress schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressSchedule {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub key_id: String,
    pub burst: i64,
    pub ratio: f64,
    pub status: EgressScheduleStatus,
}

impl EgressSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressScheduleStatus) -> bool {
        match self.status {
            EgressScheduleStatus::Requested => {
                match next {
                    EgressScheduleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressScheduleStatus::Granted => {
                return next == EgressScheduleStatus::Revoked;
            }
            EgressScheduleStatus::Revoked => {
                return next == EgressScheduleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "egress schedule", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.burst as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressSchedule {
        EgressSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            key_id: self.key_id.clone(),
            burst: self.burst,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a egress schedule in status requested.
pub fn new_egress_schedule(code: &str, name: &str) -> EgressSchedule {
    EgressSchedule {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        key_id: "".to_string(),
        burst: 0,
        ratio: 0.0,
        status: EgressScheduleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_egress_schedule("code-741", "name-542");
        assert_eq!(item.status, EgressScheduleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_egress_schedule("code-741", "name-542");
        if item.move_to(EgressScheduleStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressScheduleStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_egress_schedule("code-741", "name-542");
        item.status = EgressScheduleStatus::Expired;
        assert!(
            item.move_to(EgressScheduleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
