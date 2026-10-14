use super::*;

use acme_kit::text;

/// Where a alert cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertCycleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AlertCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AlertCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One alert cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertCycle {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub title: String,
    pub phone: String,
    pub name: String,
    pub status: AlertCycleStatus,
}

impl AlertCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertCycleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertCycleStatus) -> bool {
        match self.status {
            AlertCycleStatus::Created => {
                match next {
                    AlertCycleStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertCycleStatus::Validated => {
                return next == AlertCycleStatus::Published;
            }
            AlertCycleStatus::Published => {
                return next == AlertCycleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> AlertCycle {
        AlertCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            title: self.title.clone(),
            phone: self.phone.clone(),
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a alert cycle in status created.
pub fn new_alert_cycle(code: &str, label: &str) -> AlertCycle {
    AlertCycle {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        title: "".to_string(),
        phone: "".to_string(),
        name: "".to_string(),
        status: AlertCycleStatus::Created,
    }
}
