use super::*;

use acme_kit::text;

/// Where a default canary allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultCanaryAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultCanaryAllocationStatus {
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

impl std::fmt::Display for DefaultCanaryAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default canary allocation as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCanaryAllocation {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub duration_minutes: i64,
    pub sequence: i64,
    pub margin: f64,
    pub timezone: String,
    pub namespace: String,
    pub status: DefaultCanaryAllocationStatus,
}

impl DefaultCanaryAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultCanaryAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultCanaryAllocationStatus) -> bool {
        match self.status {
            DefaultCanaryAllocationStatus::Draft => {
                match next {
                    DefaultCanaryAllocationStatus::Pending | DefaultCanaryAllocationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultCanaryAllocationStatus::Pending => {
                return next == DefaultCanaryAllocationStatus::Suspended;
            }
            DefaultCanaryAllocationStatus::Active => {
                match next {
                    DefaultCanaryAllocationStatus::Suspended | DefaultCanaryAllocationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultCanaryAllocationStatus::Suspended => {
                match next {
                    DefaultCanaryAllocationStatus::Closed => {
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

    pub fn move_to(&mut self, next: DefaultCanaryAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default canary allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultCanaryAllocation {
        DefaultCanaryAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            duration_minutes: self.duration_minutes,
            sequence: self.sequence,
            margin: self.margin,
            timezone: self.timezone.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a default canary allocation in status draft.
pub fn new_default_canary_allocation(code: &str, label: &str) -> DefaultCanaryAllocation {
    DefaultCanaryAllocation {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        duration_minutes: 0,
        sequence: 0,
        margin: 0.0,
        timezone: "".to_string(),
        namespace: "".to_string(),
        status: DefaultCanaryAllocationStatus::Draft,
    }
}
