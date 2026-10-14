use super::*;

use acme_kit::text;

/// Where a fallback failover budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackFailoverBudgetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackFailoverBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for FallbackFailoverBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback failover budget as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverBudget {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub expires_at: i64,
    pub burst: i64,
    pub created_at: i64,
    pub weight: f64,
    pub key_id: String,
    pub status: FallbackFailoverBudgetStatus,
}

impl FallbackFailoverBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackFailoverBudgetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackFailoverBudgetStatus) -> bool {
        match self.status {
            FallbackFailoverBudgetStatus::New => {
                match next {
                    FallbackFailoverBudgetStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFailoverBudgetStatus::Queued => {
                match next {
                    FallbackFailoverBudgetStatus::Done | FallbackFailoverBudgetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFailoverBudgetStatus::Running => {
                return next == FallbackFailoverBudgetStatus::Failed;
            }
            FallbackFailoverBudgetStatus::Done => {
                match next {
                    FallbackFailoverBudgetStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackFailoverBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback failover budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.rate + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackFailoverBudget {
        FallbackFailoverBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            expires_at: self.expires_at,
            burst: self.burst,
            created_at: self.created_at,
            weight: self.weight,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback failover budget in status new.
pub fn new_fallback_failover_budget(code: &str, name: &str) -> FallbackFailoverBudget {
    FallbackFailoverBudget {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        expires_at: 0,
        burst: 0,
        created_at: 0,
        weight: 0.0,
        key_id: "".to_string(),
        status: FallbackFailoverBudgetStatus::New,
    }
}
