use super::*;

use acme_kit::text;

/// Where a freeze schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeScheduleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FreezeScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FreezeScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze schedule as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSchedule {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub summary: String,
    pub category: String,
    pub audience: String,
    pub quantity: i64,
    pub status: FreezeScheduleStatus,
}

impl FreezeSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeScheduleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeScheduleStatus) -> bool {
        match self.status {
            FreezeScheduleStatus::Created => {
                return next == FreezeScheduleStatus::Validated;
            }
            FreezeScheduleStatus::Validated => {
                match next {
                    FreezeScheduleStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeScheduleStatus::Published => {
                return next == FreezeScheduleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> FreezeSchedule {
        FreezeSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            summary: self.summary.clone(),
            category: self.category.clone(),
            audience: self.audience.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a freeze schedule in status created.
pub fn new_freeze_schedule(code: &str, title: &str) -> FreezeSchedule {
    FreezeSchedule {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        summary: "".to_string(),
        category: "".to_string(),
        audience: "".to_string(),
        quantity: 0,
        status: FreezeScheduleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_freeze_schedule("code-274", "title-914");
        assert_eq!(item.status, FreezeScheduleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_freeze_schedule("code-274", "title-914");
        if item.move_to(FreezeScheduleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeScheduleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_freeze_schedule("code-274", "title-914");
        item.status = FreezeScheduleStatus::Retired;
        assert!(
            item.move_to(FreezeScheduleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
