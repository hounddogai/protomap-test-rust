use super::*;

use acme_kit::text;

/// Where a global token override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalTokenOverrideStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalTokenOverrideStatus {
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

impl std::fmt::Display for GlobalTokenOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global token override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenOverride {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub burst: i64,
    pub country: String,
    pub quantity: i64,
    pub status: GlobalTokenOverrideStatus,
}

impl GlobalTokenOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalTokenOverrideStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalTokenOverrideStatus) -> bool {
        match self.status {
            GlobalTokenOverrideStatus::Draft => {
                return next == GlobalTokenOverrideStatus::Pending || next == GlobalTokenOverrideStatus::Active;
            }
            GlobalTokenOverrideStatus::Pending => {
                match next {
                    GlobalTokenOverrideStatus::Suspended | GlobalTokenOverrideStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTokenOverrideStatus::Active => {
                match next {
                    GlobalTokenOverrideStatus::Suspended | GlobalTokenOverrideStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTokenOverrideStatus::Suspended => {
                return next == GlobalTokenOverrideStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalTokenOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global token override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.quantity as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalTokenOverride {
        GlobalTokenOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            burst: self.burst,
            country: self.country.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a global token override in status draft.
pub fn new_global_token_override(code: &str, label: &str) -> GlobalTokenOverride {
    GlobalTokenOverride {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        burst: 0,
        country: "".to_string(),
        quantity: 0,
        status: GlobalTokenOverrideStatus::Draft,
    }
}
