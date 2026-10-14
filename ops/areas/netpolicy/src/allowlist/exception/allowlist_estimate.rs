use super::*;

use acme_kit::text;

/// Where a allowlist estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistEstimateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AllowlistEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AllowlistEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One allowlist estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEstimate {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub starts_at: i64,
    pub quantity: i64,
    pub status: AllowlistEstimateStatus,
}

impl AllowlistEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistEstimateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistEstimateStatus) -> bool {
        match self.status {
            AllowlistEstimateStatus::Proposed => {
                return next == AllowlistEstimateStatus::Approved;
            }
            AllowlistEstimateStatus::Approved => {
                match next {
                    AllowlistEstimateStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistEstimateStatus::Rejected => {
                return next == AllowlistEstimateStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist estimate {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.starts_at as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistEstimate {
        AllowlistEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            starts_at: self.starts_at,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a allowlist estimate in status proposed.
pub fn new_allowlist_estimate(code: &str, title: &str) -> AllowlistEstimate {
    AllowlistEstimate {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        starts_at: 0,
        quantity: 0,
        status: AllowlistEstimateStatus::Proposed,
    }
}
