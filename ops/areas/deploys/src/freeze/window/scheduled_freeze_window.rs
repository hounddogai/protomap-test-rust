use super::*;

use acme_kit::text;

/// Where a scheduled freeze window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledFreezeWindowStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledFreezeWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledFreezeWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled freeze window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeWindow {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub visible: bool,
    pub amount: f64,
    pub sequence: i64,
    pub capacity: i64,
    pub batch_size: i64,
    pub locale: String,
    pub margin: f64,
    pub status: ScheduledFreezeWindowStatus,
}

impl ScheduledFreezeWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledFreezeWindowStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledFreezeWindowStatus) -> bool {
        match self.status {
            ScheduledFreezeWindowStatus::Created => {
                match next {
                    ScheduledFreezeWindowStatus::Validated | ScheduledFreezeWindowStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFreezeWindowStatus::Validated => {
                return next == ScheduledFreezeWindowStatus::Retired;
            }
            ScheduledFreezeWindowStatus::Published => {
                match next {
                    ScheduledFreezeWindowStatus::Retired => {
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

    pub fn move_to(&mut self, next: ScheduledFreezeWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled freeze window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.capacity as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledFreezeWindow {
        ScheduledFreezeWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            visible: self.visible,
            amount: self.amount,
            sequence: self.sequence,
            capacity: self.capacity,
            batch_size: self.batch_size,
            locale: self.locale.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a scheduled freeze window in status created.
pub fn new_scheduled_freeze_window(code: &str, label: &str) -> ScheduledFreezeWindow {
    ScheduledFreezeWindow {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        visible: false,
        amount: 0.0,
        sequence: 0,
        capacity: 0,
        batch_size: 0,
        locale: "".to_string(),
        margin: 0.0,
        status: ScheduledFreezeWindowStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_scheduled_freeze_window("code-797", "label-172");
        assert_eq!(item.status, ScheduledFreezeWindowStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_scheduled_freeze_window("code-797", "label-172");
        if item.move_to(ScheduledFreezeWindowStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledFreezeWindowStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_scheduled_freeze_window("code-797", "label-172");
        item.status = ScheduledFreezeWindowStatus::Retired;
        assert!(
            item.move_to(ScheduledFreezeWindowStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
