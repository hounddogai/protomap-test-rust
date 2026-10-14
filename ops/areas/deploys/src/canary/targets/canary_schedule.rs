use super::*;

use acme_kit::text;

/// Where a canary schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryScheduleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CanaryScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CanaryScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a canary schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CanarySchedule {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub rotation_days: i64,
    pub cipher: String,
    pub attempts: i64,
    pub description: String,
    pub duration_minutes: i64,
    pub status: CanaryScheduleStatus,
}

impl CanarySchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryScheduleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryScheduleStatus) -> bool {
        match self.status {
            CanaryScheduleStatus::Proposed => {
                return next == CanaryScheduleStatus::Approved;
            }
            CanaryScheduleStatus::Approved => {
                match next {
                    CanaryScheduleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryScheduleStatus::Rejected => {
                return next == CanaryScheduleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary schedule {} cannot move from {} to {}",
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
        format!("{} {}", "canary schedule", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.attempts as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanarySchedule {
        CanarySchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            rotation_days: self.rotation_days,
            cipher: self.cipher.clone(),
            attempts: self.attempts,
            description: self.description.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a canary schedule in status proposed.
pub fn new_canary_schedule(code: &str, label: &str) -> CanarySchedule {
    CanarySchedule {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        rotation_days: 0,
        cipher: "".to_string(),
        attempts: 0,
        description: "".to_string(),
        duration_minutes: 0,
        status: CanaryScheduleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_canary_schedule("code-103", "label-85");
        assert_eq!(item.status, CanaryScheduleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_canary_schedule("code-103", "label-85");
        if item.move_to(CanaryScheduleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryScheduleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_canary_schedule("code-103", "label-85");
        item.status = CanaryScheduleStatus::Withdrawn;
        assert!(
            item.move_to(CanaryScheduleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
