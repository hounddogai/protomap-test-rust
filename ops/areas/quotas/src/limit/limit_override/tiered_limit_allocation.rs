use super::*;

use acme_kit::text;

/// Where a tiered limit allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredLimitAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredLimitAllocationStatus {
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

impl std::fmt::Display for TieredLimitAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered limit allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitAllocation {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub ends_at: i64,
    pub maximum: f64,
    pub revision: i64,
    pub approved: bool,
    pub source: String,
    pub category: String,
    pub port: i64,
    pub status: TieredLimitAllocationStatus,
}

impl TieredLimitAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredLimitAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredLimitAllocationStatus) -> bool {
        match self.status {
            TieredLimitAllocationStatus::Draft => {
                return next == TieredLimitAllocationStatus::Closed;
            }
            TieredLimitAllocationStatus::Pending => {
                return next == TieredLimitAllocationStatus::Suspended || next == TieredLimitAllocationStatus::Closed;
            }
            TieredLimitAllocationStatus::Active => {
                return next == TieredLimitAllocationStatus::Suspended || next == TieredLimitAllocationStatus::Closed;
            }
            TieredLimitAllocationStatus::Suspended => {
                return next == TieredLimitAllocationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredLimitAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered limit allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.ends_at as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredLimitAllocation {
        TieredLimitAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            ends_at: self.ends_at,
            maximum: self.maximum,
            revision: self.revision,
            approved: self.approved,
            source: self.source.clone(),
            category: self.category.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a tiered limit allocation in status draft.
pub fn new_tiered_limit_allocation(code: &str, name: &str) -> TieredLimitAllocation {
    TieredLimitAllocation {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        ends_at: 0,
        maximum: 0.0,
        revision: 0,
        approved: false,
        source: "".to_string(),
        category: "".to_string(),
        port: 0,
        status: TieredLimitAllocationStatus::Draft,
    }
}
