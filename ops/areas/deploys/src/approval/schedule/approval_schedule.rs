use super::*;

use acme_kit::text;

/// Where a approval schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ApprovalScheduleStatus {
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

impl std::fmt::Display for ApprovalScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSchedule {
    pub code: String,
    pub label: String,
    pub name: String,
    pub amount: f64,
    pub fingerprint: String,
    pub unit: String,
    pub country: String,
    pub quantity: i64,
    pub reviewer: String,
    pub host: String,
    pub status: ApprovalScheduleStatus,
}

impl ApprovalSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalScheduleStatus) -> bool {
        match self.status {
            ApprovalScheduleStatus::Idle => {
                return next == ApprovalScheduleStatus::Stopped;
            }
            ApprovalScheduleStatus::Warming => {
                return next == ApprovalScheduleStatus::Draining;
            }
            ApprovalScheduleStatus::Ready => {
                return next == ApprovalScheduleStatus::Draining;
            }
            ApprovalScheduleStatus::Draining => {
                match next {
                    ApprovalScheduleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ApprovalScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.amount * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalSchedule {
        ApprovalSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            amount: self.amount,
            fingerprint: self.fingerprint.clone(),
            unit: self.unit.clone(),
            country: self.country.clone(),
            quantity: self.quantity,
            reviewer: self.reviewer.clone(),
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a approval schedule in status idle.
pub fn new_approval_schedule(code: &str, label: &str) -> ApprovalSchedule {
    ApprovalSchedule {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        amount: 0.0,
        fingerprint: "".to_string(),
        unit: "".to_string(),
        country: "".to_string(),
        quantity: 0,
        reviewer: "".to_string(),
        host: "".to_string(),
        status: ApprovalScheduleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_approval_schedule("code-211", "label-763");
        assert_eq!(item.status, ApprovalScheduleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_approval_schedule("code-211", "label-763");
        if item.move_to(ApprovalScheduleStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalScheduleStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_approval_schedule("code-211", "label-763");
        item.status = ApprovalScheduleStatus::Stopped;
        assert!(
            item.move_to(ApprovalScheduleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
