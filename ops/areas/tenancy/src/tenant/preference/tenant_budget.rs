use super::*;

use acme_kit::text;

/// Where a tenant budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantBudgetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TenantBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TenantBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tenant budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantBudget {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub external_ref: String,
    pub timezone: String,
    pub name: String,
    pub capacity: i64,
    pub source: String,
    pub starts_at: i64,
    pub ends_at: i64,
    pub status: TenantBudgetStatus,
}

impl TenantBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantBudgetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantBudgetStatus) -> bool {
        match self.status {
            TenantBudgetStatus::Planned => {
                return next == TenantBudgetStatus::Scheduled || next == TenantBudgetStatus::InProgress;
            }
            TenantBudgetStatus::Scheduled => {
                return next == TenantBudgetStatus::Completed || next == TenantBudgetStatus::Cancelled;
            }
            TenantBudgetStatus::InProgress => {
                return next == TenantBudgetStatus::Cancelled;
            }
            TenantBudgetStatus::Completed => {
                return next == TenantBudgetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ends_at as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantBudget {
        TenantBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            external_ref: self.external_ref.clone(),
            timezone: self.timezone.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            source: self.source.clone(),
            starts_at: self.starts_at,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a tenant budget in status planned.
pub fn new_tenant_budget(code: &str, title: &str) -> TenantBudget {
    TenantBudget {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        external_ref: "".to_string(),
        timezone: "".to_string(),
        name: "".to_string(),
        capacity: 0,
        source: "".to_string(),
        starts_at: 0,
        ends_at: 0,
        status: TenantBudgetStatus::Planned,
    }
}
