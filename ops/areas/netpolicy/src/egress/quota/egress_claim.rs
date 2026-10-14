use super::*;

use acme_kit::text;

/// Where a egress claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressClaimStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EgressClaimStatus {
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

impl std::fmt::Display for EgressClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a egress claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressClaim {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub duration_minutes: i64,
    pub threshold: f64,
    pub ends_at: i64,
    pub limit: i64,
    pub status: EgressClaimStatus,
}

impl EgressClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressClaimStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressClaimStatus) -> bool {
        match self.status {
            EgressClaimStatus::New => {
                match next {
                    EgressClaimStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressClaimStatus::Queued => {
                match next {
                    EgressClaimStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressClaimStatus::Running => {
                match next {
                    EgressClaimStatus::Done | EgressClaimStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressClaimStatus::Done => {
                return next == EgressClaimStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress claim {} cannot move from {} to {}",
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
        format!("{} {}", "egress claim", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.limit as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressClaim {
        EgressClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            principal: self.principal.clone(),
            duration_minutes: self.duration_minutes,
            threshold: self.threshold,
            ends_at: self.ends_at,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a egress claim in status new.
pub fn new_egress_claim(code: &str, label: &str) -> EgressClaim {
    EgressClaim {
        code: code.to_string(),
        label: label.to_string(),
        principal: "".to_string(),
        duration_minutes: 0,
        threshold: 0.0,
        ends_at: 0,
        limit: 0,
        status: EgressClaimStatus::New,
    }
}
