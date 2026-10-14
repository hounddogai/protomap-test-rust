use super::*;

use acme_kit::text;

/// Where a lenient retention claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRetentionClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientRetentionClaimStatus {
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

impl std::fmt::Display for LenientRetentionClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient retention claim as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionClaim {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub threshold: f64,
    pub budget: f64,
    pub status: LenientRetentionClaimStatus,
}

impl LenientRetentionClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRetentionClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRetentionClaimStatus) -> bool {
        match self.status {
            LenientRetentionClaimStatus::Draft => {
                return next == LenientRetentionClaimStatus::Closed;
            }
            LenientRetentionClaimStatus::Pending => {
                match next {
                    LenientRetentionClaimStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRetentionClaimStatus::Active => {
                match next {
                    LenientRetentionClaimStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRetentionClaimStatus::Suspended => {
                match next {
                    LenientRetentionClaimStatus::Closed => {
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

    pub fn move_to(&mut self, next: LenientRetentionClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient retention claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient retention claim", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.grace_days as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRetentionClaim {
        LenientRetentionClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            threshold: self.threshold,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient retention claim in status draft.
pub fn new_lenient_retention_claim(code: &str, label: &str) -> LenientRetentionClaim {
    LenientRetentionClaim {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        threshold: 0.0,
        budget: 0.0,
        status: LenientRetentionClaimStatus::Draft,
    }
}
