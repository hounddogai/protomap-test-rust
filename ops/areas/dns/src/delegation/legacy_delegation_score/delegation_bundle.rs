use super::*;

use acme_kit::text;

/// Where a delegation bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationBundleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DelegationBundleStatus {
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

impl std::fmt::Display for DelegationBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBundle {
    pub code: String,
    pub label: String,
    pub description: String,
    pub quantity: i64,
    pub window_days: i64,
    pub quota_units: i64,
    pub status: DelegationBundleStatus,
}

impl DelegationBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationBundleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationBundleStatus) -> bool {
        match self.status {
            DelegationBundleStatus::Draft => {
                return next == DelegationBundleStatus::Closed;
            }
            DelegationBundleStatus::Pending => {
                return next == DelegationBundleStatus::Suspended || next == DelegationBundleStatus::Closed;
            }
            DelegationBundleStatus::Active => {
                match next {
                    DelegationBundleStatus::Suspended | DelegationBundleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationBundleStatus::Suspended => {
                return next == DelegationBundleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.quantity as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationBundle {
        DelegationBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            quantity: self.quantity,
            window_days: self.window_days,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a delegation bundle in status draft.
pub fn new_delegation_bundle(code: &str, label: &str) -> DelegationBundle {
    DelegationBundle {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        quantity: 0,
        window_days: 0,
        quota_units: 0,
        status: DelegationBundleStatus::Draft,
    }
}
