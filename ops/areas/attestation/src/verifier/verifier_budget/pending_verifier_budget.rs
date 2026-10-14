use super::*;

use acme_kit::text;

/// Where a pending verifier budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingVerifierBudgetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingVerifierBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PendingVerifierBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending verifier budget as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierBudget {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub ttl_seconds: i64,
    pub updated_at: i64,
    pub threshold: f64,
    pub locale: String,
    pub status: PendingVerifierBudgetStatus,
}

impl PendingVerifierBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingVerifierBudgetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingVerifierBudgetStatus) -> bool {
        match self.status {
            PendingVerifierBudgetStatus::Open => {
                match next {
                    PendingVerifierBudgetStatus::Resolved | PendingVerifierBudgetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingVerifierBudgetStatus::InReview => {
                return next == PendingVerifierBudgetStatus::Resolved || next == PendingVerifierBudgetStatus::Archived;
            }
            PendingVerifierBudgetStatus::Resolved => {
                match next {
                    PendingVerifierBudgetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingVerifierBudgetStatus::Reopened => {
                match next {
                    PendingVerifierBudgetStatus::Archived => {
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

    pub fn move_to(&mut self, next: PendingVerifierBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending verifier budget {} cannot move from {} to {}",
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
        format!("{} {}", "pending verifier budget", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.updated_at as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingVerifierBudget {
        PendingVerifierBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            ttl_seconds: self.ttl_seconds,
            updated_at: self.updated_at,
            threshold: self.threshold,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a pending verifier budget in status open.
pub fn new_pending_verifier_budget(code: &str, name: &str) -> PendingVerifierBudget {
    PendingVerifierBudget {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        ttl_seconds: 0,
        updated_at: 0,
        threshold: 0.0,
        locale: "".to_string(),
        status: PendingVerifierBudgetStatus::Open,
    }
}
