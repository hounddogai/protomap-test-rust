use super::*;

use acme_kit::text;

/// Where a primary policy item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryPolicyItemStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryPolicyItemStatus {
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

impl std::fmt::Display for PrimaryPolicyItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary policy item and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPolicyItem {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub source: String,
    pub quota_units: i64,
    pub priority: i64,
    pub status: PrimaryPolicyItemStatus,
}

impl PrimaryPolicyItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryPolicyItemStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryPolicyItemStatus) -> bool {
        match self.status {
            PrimaryPolicyItemStatus::Draft => {
                match next {
                    PrimaryPolicyItemStatus::Active | PrimaryPolicyItemStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPolicyItemStatus::Pending => {
                return next == PrimaryPolicyItemStatus::Active || next == PrimaryPolicyItemStatus::Suspended;
            }
            PrimaryPolicyItemStatus::Active => {
                return next == PrimaryPolicyItemStatus::Suspended || next == PrimaryPolicyItemStatus::Closed;
            }
            PrimaryPolicyItemStatus::Suspended => {
                return next == PrimaryPolicyItemStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryPolicyItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary policy item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary policy item", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.quota_units as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryPolicyItem {
        PrimaryPolicyItem {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            source: self.source.clone(),
            quota_units: self.quota_units,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary policy item in status draft.
pub fn new_primary_policy_item(code: &str, name: &str) -> PrimaryPolicyItem {
    PrimaryPolicyItem {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        source: "".to_string(),
        quota_units: 0,
        priority: 0,
        status: PrimaryPolicyItemStatus::Draft,
    }
}
