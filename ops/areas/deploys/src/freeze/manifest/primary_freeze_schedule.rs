use super::*;

use acme_kit::text;

/// Where a primary freeze schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryFreezeScheduleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryFreezeScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryFreezeScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary freeze schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFreezeSchedule {
    pub code: String,
    pub label: String,
    pub ratio: f64,
    pub source: String,
    pub fingerprint: String,
    pub status: PrimaryFreezeScheduleStatus,
}

impl PrimaryFreezeSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryFreezeScheduleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryFreezeScheduleStatus) -> bool {
        match self.status {
            PrimaryFreezeScheduleStatus::Created => {
                return next == PrimaryFreezeScheduleStatus::Validated
                    || next == PrimaryFreezeScheduleStatus::Published;
            }
            PrimaryFreezeScheduleStatus::Validated => {
                return next == PrimaryFreezeScheduleStatus::Published || next == PrimaryFreezeScheduleStatus::Retired;
            }
            PrimaryFreezeScheduleStatus::Published => {
                match next {
                    PrimaryFreezeScheduleStatus::Retired => {
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

    pub fn move_to(&mut self, next: PrimaryFreezeScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary freeze schedule {} cannot move from {} to {}",
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
        self.ratio
    }

    pub fn copy(&self) -> PrimaryFreezeSchedule {
        PrimaryFreezeSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            source: self.source.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary freeze schedule in status created.
pub fn new_primary_freeze_schedule(code: &str, label: &str) -> PrimaryFreezeSchedule {
    PrimaryFreezeSchedule {
        code: code.to_string(),
        label: label.to_string(),
        ratio: 0.0,
        source: "".to_string(),
        fingerprint: "".to_string(),
        status: PrimaryFreezeScheduleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_primary_freeze_schedule("code-983", "label-851");
        assert_eq!(item.status, PrimaryFreezeScheduleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_primary_freeze_schedule("code-983", "label-851");
        if item.move_to(PrimaryFreezeScheduleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryFreezeScheduleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_primary_freeze_schedule("code-983", "label-851");
        item.status = PrimaryFreezeScheduleStatus::Retired;
        assert!(
            item.move_to(PrimaryFreezeScheduleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
