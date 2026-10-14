use super::*;

use acme_kit::text;

/// Where a rollout window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutWindowStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RolloutWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RolloutWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutWindow {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub principal: String,
    pub country: String,
    pub capacity: i64,
    pub description: String,
    pub ceiling: f64,
    pub status: RolloutWindowStatus,
}

impl RolloutWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutWindowStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutWindowStatus) -> bool {
        match self.status {
            RolloutWindowStatus::Planned => {
                match next {
                    RolloutWindowStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutWindowStatus::Scheduled => {
                return next == RolloutWindowStatus::Completed || next == RolloutWindowStatus::Cancelled;
            }
            RolloutWindowStatus::InProgress => {
                return next == RolloutWindowStatus::Completed || next == RolloutWindowStatus::Cancelled;
            }
            RolloutWindowStatus::Completed => {
                return next == RolloutWindowStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.capacity as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutWindow {
        RolloutWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            principal: self.principal.clone(),
            country: self.country.clone(),
            capacity: self.capacity,
            description: self.description.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rollout window in status planned.
pub fn new_rollout_window(code: &str, name: &str) -> RolloutWindow {
    RolloutWindow {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        principal: "".to_string(),
        country: "".to_string(),
        capacity: 0,
        description: "".to_string(),
        ceiling: 0.0,
        status: RolloutWindowStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rollout_window("code-340", "name-978");
        assert_eq!(item.status, RolloutWindowStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rollout_window("code-340", "name-978");
        if item.move_to(RolloutWindowStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutWindowStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rollout_window("code-340", "name-978");
        item.status = RolloutWindowStatus::Cancelled;
        assert!(
            item.move_to(RolloutWindowStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
