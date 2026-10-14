use super::*;

use acme_kit::text;

/// Where a seasonal token target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalTokenTargetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalTokenTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalTokenTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal token target as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenTarget {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub scope: String,
    pub country: String,
    pub owner_id: String,
    pub active: bool,
    pub quantity: i64,
    pub starts_at: i64,
    pub status: SeasonalTokenTargetStatus,
}

impl SeasonalTokenTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalTokenTargetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalTokenTargetStatus) -> bool {
        match self.status {
            SeasonalTokenTargetStatus::Proposed => {
                return next == SeasonalTokenTargetStatus::Approved || next == SeasonalTokenTargetStatus::Rejected;
            }
            SeasonalTokenTargetStatus::Approved => {
                match next {
                    SeasonalTokenTargetStatus::Rejected | SeasonalTokenTargetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTokenTargetStatus::Rejected => {
                return next == SeasonalTokenTargetStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalTokenTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal token target {} cannot move from {} to {}",
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

    pub fn adjusted_load(&self) -> f64 {
        ((self.quantity as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalTokenTarget {
        SeasonalTokenTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            scope: self.scope.clone(),
            country: self.country.clone(),
            owner_id: self.owner_id.clone(),
            active: self.active,
            quantity: self.quantity,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a seasonal token target in status proposed.
pub fn new_seasonal_token_target(code: &str, title: &str) -> SeasonalTokenTarget {
    SeasonalTokenTarget {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        scope: "".to_string(),
        country: "".to_string(),
        owner_id: "".to_string(),
        active: false,
        quantity: 0,
        starts_at: 0,
        status: SeasonalTokenTargetStatus::Proposed,
    }
}
