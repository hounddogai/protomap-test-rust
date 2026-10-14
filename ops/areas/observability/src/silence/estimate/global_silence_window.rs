use super::*;

use acme_kit::text;

/// Where a global silence window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSilenceWindowStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalSilenceWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalSilenceWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global silence window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceWindow {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub notes: String,
    pub country: String,
    pub active: bool,
    pub quantity: i64,
    pub status: GlobalSilenceWindowStatus,
}

impl GlobalSilenceWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSilenceWindowStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSilenceWindowStatus) -> bool {
        match self.status {
            GlobalSilenceWindowStatus::Created => {
                return next == GlobalSilenceWindowStatus::Published;
            }
            GlobalSilenceWindowStatus::Validated => {
                match next {
                    GlobalSilenceWindowStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSilenceWindowStatus::Published => {
                match next {
                    GlobalSilenceWindowStatus::Retired => {
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

    pub fn move_to(&mut self, next: GlobalSilenceWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global silence window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global silence window", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> GlobalSilenceWindow {
        GlobalSilenceWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            notes: self.notes.clone(),
            country: self.country.clone(),
            active: self.active,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a global silence window in status created.
pub fn new_global_silence_window(code: &str, label: &str) -> GlobalSilenceWindow {
    GlobalSilenceWindow {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        notes: "".to_string(),
        country: "".to_string(),
        active: false,
        quantity: 0,
        status: GlobalSilenceWindowStatus::Created,
    }
}
