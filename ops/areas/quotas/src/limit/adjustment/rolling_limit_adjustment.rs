use super::*;

use acme_kit::text;

/// Where a rolling limit adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingLimitAdjustmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingLimitAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingLimitAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling limit adjustment as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitAdjustment {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub retries: i64,
    pub ratio: f64,
    pub checksum: String,
    pub country: String,
    pub quantity: i64,
    pub audience: String,
    pub status: RollingLimitAdjustmentStatus,
}

impl RollingLimitAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingLimitAdjustmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingLimitAdjustmentStatus) -> bool {
        match self.status {
            RollingLimitAdjustmentStatus::Proposed => {
                return next == RollingLimitAdjustmentStatus::Rejected
                    || next == RollingLimitAdjustmentStatus::Withdrawn;
            }
            RollingLimitAdjustmentStatus::Approved => {
                match next {
                    RollingLimitAdjustmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingLimitAdjustmentStatus::Rejected => {
                return next == RollingLimitAdjustmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingLimitAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling limit adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.quantity as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingLimitAdjustment {
        RollingLimitAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            retries: self.retries,
            ratio: self.ratio,
            checksum: self.checksum.clone(),
            country: self.country.clone(),
            quantity: self.quantity,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rolling limit adjustment in status proposed.
pub fn new_rolling_limit_adjustment(code: &str, title: &str) -> RollingLimitAdjustment {
    RollingLimitAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        retries: 0,
        ratio: 0.0,
        checksum: "".to_string(),
        country: "".to_string(),
        quantity: 0,
        audience: "".to_string(),
        status: RollingLimitAdjustmentStatus::Proposed,
    }
}
