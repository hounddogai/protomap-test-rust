use super::*;

use acme_kit::text;

/// Where a binding rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BindingRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BindingRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingRollup {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub attempts: i64,
    pub source: String,
    pub external_ref: String,
    pub margin: f64,
    pub status: BindingRollupStatus,
}

impl BindingRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingRollupStatus) -> bool {
        match self.status {
            BindingRollupStatus::Proposed => {
                return next == BindingRollupStatus::Approved || next == BindingRollupStatus::Rejected;
            }
            BindingRollupStatus::Approved => {
                match next {
                    BindingRollupStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingRollupStatus::Rejected => {
                match next {
                    BindingRollupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: BindingRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.minimum - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingRollup {
        BindingRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            attempts: self.attempts,
            source: self.source.clone(),
            external_ref: self.external_ref.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a binding rollup in status proposed.
pub fn new_binding_rollup(code: &str, name: &str) -> BindingRollup {
    BindingRollup {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        attempts: 0,
        source: "".to_string(),
        external_ref: "".to_string(),
        margin: 0.0,
        status: BindingRollupStatus::Proposed,
    }
}
