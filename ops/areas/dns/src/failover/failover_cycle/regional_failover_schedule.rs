use super::*;

use acme_kit::text;

/// Where a regional failover schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalFailoverScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalFailoverScheduleStatus {
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

impl std::fmt::Display for RegionalFailoverScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional failover schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverSchedule {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub title: String,
    pub visible: bool,
    pub amount: f64,
    pub status: RegionalFailoverScheduleStatus,
}

impl RegionalFailoverSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalFailoverScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalFailoverScheduleStatus) -> bool {
        match self.status {
            RegionalFailoverScheduleStatus::Draft => {
                match next {
                    RegionalFailoverScheduleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFailoverScheduleStatus::Pending => {
                match next {
                    RegionalFailoverScheduleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFailoverScheduleStatus::Active => {
                match next {
                    RegionalFailoverScheduleStatus::Suspended | RegionalFailoverScheduleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFailoverScheduleStatus::Suspended => {
                return next == RegionalFailoverScheduleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalFailoverScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional failover schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional failover schedule", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> RegionalFailoverSchedule {
        RegionalFailoverSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            title: self.title.clone(),
            visible: self.visible,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional failover schedule in status draft.
pub fn new_regional_failover_schedule(code: &str, label: &str) -> RegionalFailoverSchedule {
    RegionalFailoverSchedule {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        title: "".to_string(),
        visible: false,
        amount: 0.0,
        status: RegionalFailoverScheduleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_failover_schedule("code-615", "label-596");
        assert_eq!(item.status, RegionalFailoverScheduleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_regional_failover_schedule("code-615", "label-596");
        if item.move_to(RegionalFailoverScheduleStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalFailoverScheduleStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_failover_schedule("code-615", "label-596");
        item.status = RegionalFailoverScheduleStatus::Closed;
        assert!(
            item.move_to(RegionalFailoverScheduleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
