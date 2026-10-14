use super::*;

use acme_kit::text;

/// Where a rolling dns record schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingDnsRecordScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingDnsRecordScheduleStatus {
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

impl std::fmt::Display for RollingDnsRecordScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling dns record schedule as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDnsRecordSchedule {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub contact_email: String,
    pub external_ref: String,
    pub window_days: i64,
    pub floor_value: f64,
    pub status: RollingDnsRecordScheduleStatus,
}

impl RollingDnsRecordSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingDnsRecordScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingDnsRecordScheduleStatus) -> bool {
        match self.status {
            RollingDnsRecordScheduleStatus::Idle => {
                match next {
                    RollingDnsRecordScheduleStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingDnsRecordScheduleStatus::Warming => {
                return next == RollingDnsRecordScheduleStatus::Draining;
            }
            RollingDnsRecordScheduleStatus::Ready => {
                match next {
                    RollingDnsRecordScheduleStatus::Draining | RollingDnsRecordScheduleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingDnsRecordScheduleStatus::Draining => {
                return next == RollingDnsRecordScheduleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingDnsRecordScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling dns record schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.rotation_days as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingDnsRecordSchedule {
        RollingDnsRecordSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            contact_email: self.contact_email.clone(),
            external_ref: self.external_ref.clone(),
            window_days: self.window_days,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling dns record schedule in status idle.
pub fn new_rolling_dns_record_schedule(code: &str, name: &str) -> RollingDnsRecordSchedule {
    RollingDnsRecordSchedule {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        contact_email: "".to_string(),
        external_ref: "".to_string(),
        window_days: 0,
        floor_value: 0.0,
        status: RollingDnsRecordScheduleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rolling_dns_record_schedule("code-86", "name-262");
        assert_eq!(item.status, RollingDnsRecordScheduleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_rolling_dns_record_schedule("code-86", "name-262");
        if item.move_to(RollingDnsRecordScheduleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingDnsRecordScheduleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rolling_dns_record_schedule("code-86", "name-262");
        item.status = RollingDnsRecordScheduleStatus::Stopped;
        assert!(
            item.move_to(RollingDnsRecordScheduleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
