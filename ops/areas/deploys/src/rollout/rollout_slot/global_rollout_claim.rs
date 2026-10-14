use super::*;

use acme_kit::text;

/// Where a global rollout claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRolloutClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GlobalRolloutClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GlobalRolloutClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global rollout claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutClaim {
    pub code: String,
    pub title: String,
    pub country: String,
    pub duration_minutes: i64,
    pub window_days: i64,
    pub summary: String,
    pub discount: f64,
    pub timezone: String,
    pub status: GlobalRolloutClaimStatus,
}

impl GlobalRolloutClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRolloutClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRolloutClaimStatus) -> bool {
        match self.status {
            GlobalRolloutClaimStatus::Proposed => {
                match next {
                    GlobalRolloutClaimStatus::Approved | GlobalRolloutClaimStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRolloutClaimStatus::Approved => {
                return next == GlobalRolloutClaimStatus::Rejected || next == GlobalRolloutClaimStatus::Withdrawn;
            }
            GlobalRolloutClaimStatus::Rejected => {
                return next == GlobalRolloutClaimStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalRolloutClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global rollout claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.discount + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalRolloutClaim {
        GlobalRolloutClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            duration_minutes: self.duration_minutes,
            window_days: self.window_days,
            summary: self.summary.clone(),
            discount: self.discount,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a global rollout claim in status proposed.
pub fn new_global_rollout_claim(code: &str, title: &str) -> GlobalRolloutClaim {
    GlobalRolloutClaim {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        duration_minutes: 0,
        window_days: 0,
        summary: "".to_string(),
        discount: 0.0,
        timezone: "".to_string(),
        status: GlobalRolloutClaimStatus::Proposed,
    }
}
