use super::*;

use acme_kit::text;

/// Where a policy window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyWindowStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PolicyWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PolicyWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyWindow {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub quantity: i64,
    pub currency: String,
    pub capacity: i64,
    pub scope: String,
    pub cluster: String,
    pub minimum: f64,
    pub amount: f64,
    pub description: String,
    pub status: PolicyWindowStatus,
}

impl PolicyWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyWindowStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyWindowStatus) -> bool {
        match self.status {
            PolicyWindowStatus::Created => {
                match next {
                    PolicyWindowStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyWindowStatus::Validated => {
                return next == PolicyWindowStatus::Published;
            }
            PolicyWindowStatus::Published => {
                match next {
                    PolicyWindowStatus::Retired => {
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

    pub fn move_to(&mut self, next: PolicyWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "policy window", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.minimum - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyWindow {
        PolicyWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            quantity: self.quantity,
            currency: self.currency.clone(),
            capacity: self.capacity,
            scope: self.scope.clone(),
            cluster: self.cluster.clone(),
            minimum: self.minimum,
            amount: self.amount,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a policy window in status created.
pub fn new_policy_window(code: &str, label: &str) -> PolicyWindow {
    PolicyWindow {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        quantity: 0,
        currency: "".to_string(),
        capacity: 0,
        scope: "".to_string(),
        cluster: "".to_string(),
        minimum: 0.0,
        amount: 0.0,
        description: "".to_string(),
        status: PolicyWindowStatus::Created,
    }
}
