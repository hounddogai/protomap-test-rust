use super::*;

use acme_kit::text;

/// Where a default slo adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultSloAdjustmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultSloAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultSloAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default slo adjustment as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSloAdjustment {
    pub code: String,
    pub title: String,
    pub attempts: i64,
    pub source: String,
    pub expires_at: i64,
    pub amount: f64,
    pub status: DefaultSloAdjustmentStatus,
}

impl DefaultSloAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultSloAdjustmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultSloAdjustmentStatus) -> bool {
        match self.status {
            DefaultSloAdjustmentStatus::Requested => {
                return next == DefaultSloAdjustmentStatus::Granted || next == DefaultSloAdjustmentStatus::Expired;
            }
            DefaultSloAdjustmentStatus::Granted => {
                return next == DefaultSloAdjustmentStatus::Expired;
            }
            DefaultSloAdjustmentStatus::Revoked => {
                return next == DefaultSloAdjustmentStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultSloAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default slo adjustment {} cannot move from {} to {}",
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
        format!("{} {}", "default slo adjustment", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.amount - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultSloAdjustment {
        DefaultSloAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            source: self.source.clone(),
            expires_at: self.expires_at,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a default slo adjustment in status requested.
pub fn new_default_slo_adjustment(code: &str, title: &str) -> DefaultSloAdjustment {
    DefaultSloAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        attempts: 0,
        source: "".to_string(),
        expires_at: 0,
        amount: 0.0,
        status: DefaultSloAdjustmentStatus::Requested,
    }
}
