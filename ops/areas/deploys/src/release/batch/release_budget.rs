use super::*;

use acme_kit::text;

/// Where a release budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ReleaseBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ReleaseBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a release budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBudget {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub maximum: f64,
    pub issuer: String,
    pub title: String,
    pub status: ReleaseBudgetStatus,
}

impl ReleaseBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseBudgetStatus) -> bool {
        match self.status {
            ReleaseBudgetStatus::Created => {
                match next {
                    ReleaseBudgetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseBudgetStatus::Validated => {
                match next {
                    ReleaseBudgetStatus::Published | ReleaseBudgetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseBudgetStatus::Published => {
                return next == ReleaseBudgetStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> ReleaseBudget {
        ReleaseBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            maximum: self.maximum,
            issuer: self.issuer.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a release budget in status created.
pub fn new_release_budget(code: &str, label: &str) -> ReleaseBudget {
    ReleaseBudget {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        maximum: 0.0,
        issuer: "".to_string(),
        title: "".to_string(),
        status: ReleaseBudgetStatus::Created,
    }
}
