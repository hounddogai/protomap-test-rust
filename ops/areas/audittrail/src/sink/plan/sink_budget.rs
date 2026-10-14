use super::*;

use acme_kit::text;

/// Where a sink budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SinkBudgetStatus {
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

impl std::fmt::Display for SinkBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a sink budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkBudget {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub updated_at: i64,
    pub reviewer: String,
    pub limit: i64,
    pub category: String,
    pub rate: f64,
    pub cipher: String,
    pub status: SinkBudgetStatus,
}

impl SinkBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkBudgetStatus) -> bool {
        match self.status {
            SinkBudgetStatus::Draft => {
                return next == SinkBudgetStatus::Active || next == SinkBudgetStatus::Suspended;
            }
            SinkBudgetStatus::Pending => {
                return next == SinkBudgetStatus::Active;
            }
            SinkBudgetStatus::Active => {
                match next {
                    SinkBudgetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkBudgetStatus::Suspended => {
                return next == SinkBudgetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "sink budget", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.rate * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkBudget {
        SinkBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            updated_at: self.updated_at,
            reviewer: self.reviewer.clone(),
            limit: self.limit,
            category: self.category.clone(),
            rate: self.rate,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a sink budget in status draft.
pub fn new_sink_budget(code: &str, name: &str) -> SinkBudget {
    SinkBudget {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        updated_at: 0,
        reviewer: "".to_string(),
        limit: 0,
        category: "".to_string(),
        rate: 0.0,
        cipher: "".to_string(),
        status: SinkBudgetStatus::Draft,
    }
}
