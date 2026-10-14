use super::*;

use acme_kit::text;

/// Where a silence window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceWindowStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SilenceWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SilenceWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence window as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceWindow {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub description: String,
    pub issuer: String,
    pub status: SilenceWindowStatus,
}

impl SilenceWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceWindowStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceWindowStatus) -> bool {
        match self.status {
            SilenceWindowStatus::Proposed => {
                return next == SilenceWindowStatus::Approved || next == SilenceWindowStatus::Rejected;
            }
            SilenceWindowStatus::Approved => {
                match next {
                    SilenceWindowStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceWindowStatus::Rejected => {
                match next {
                    SilenceWindowStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SilenceWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "silence window", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> SilenceWindow {
        SilenceWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            description: self.description.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a silence window in status proposed.
pub fn new_silence_window(code: &str, label: &str) -> SilenceWindow {
    SilenceWindow {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        description: "".to_string(),
        issuer: "".to_string(),
        status: SilenceWindowStatus::Proposed,
    }
}
