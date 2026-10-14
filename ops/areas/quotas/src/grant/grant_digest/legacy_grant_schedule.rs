use super::*;

use acme_kit::text;

/// Where a legacy grant schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyGrantScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyGrantScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for LegacyGrantScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy grant schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantSchedule {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub region: String,
    pub ceiling: f64,
    pub status: LegacyGrantScheduleStatus,
}

impl LegacyGrantSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyGrantScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyGrantScheduleStatus) -> bool {
        match self.status {
            LegacyGrantScheduleStatus::Draft => {
                match next {
                    LegacyGrantScheduleStatus::Pending | LegacyGrantScheduleStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyGrantScheduleStatus::Pending => {
                return next == LegacyGrantScheduleStatus::Active || next == LegacyGrantScheduleStatus::Suspended;
            }
            LegacyGrantScheduleStatus::Active => {
                match next {
                    LegacyGrantScheduleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyGrantScheduleStatus::Suspended => {
                match next {
                    LegacyGrantScheduleStatus::Closed => {
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

    pub fn move_to(&mut self, next: LegacyGrantScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy grant schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ceiling + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyGrantSchedule {
        LegacyGrantSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            region: self.region.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a legacy grant schedule in status draft.
pub fn new_legacy_grant_schedule(code: &str, label: &str) -> LegacyGrantSchedule {
    LegacyGrantSchedule {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        region: "".to_string(),
        ceiling: 0.0,
        status: LegacyGrantScheduleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_grant_schedule("code-738", "label-71");
        assert_eq!(item.status, LegacyGrantScheduleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_legacy_grant_schedule("code-738", "label-71");
        if item.move_to(LegacyGrantScheduleStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyGrantScheduleStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_grant_schedule("code-738", "label-71");
        item.status = LegacyGrantScheduleStatus::Closed;
        assert!(
            item.move_to(LegacyGrantScheduleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
