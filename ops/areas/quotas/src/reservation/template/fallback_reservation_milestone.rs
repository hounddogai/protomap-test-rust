use super::*;

use acme_kit::text;

/// Where a fallback reservation milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackReservationMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackReservationMilestoneStatus {
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

impl std::fmt::Display for FallbackReservationMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback reservation milestone as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationMilestone {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub quantity: i64,
    pub source: String,
    pub locale: String,
    pub score: f64,
    pub country: String,
    pub cipher: String,
    pub host: String,
    pub limit: i64,
    pub status: FallbackReservationMilestoneStatus,
}

impl FallbackReservationMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackReservationMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackReservationMilestoneStatus) -> bool {
        match self.status {
            FallbackReservationMilestoneStatus::Draft => {
                return next == FallbackReservationMilestoneStatus::Active;
            }
            FallbackReservationMilestoneStatus::Pending => {
                match next {
                    FallbackReservationMilestoneStatus::Active | FallbackReservationMilestoneStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackReservationMilestoneStatus::Active => {
                return next == FallbackReservationMilestoneStatus::Suspended;
            }
            FallbackReservationMilestoneStatus::Suspended => {
                return next == FallbackReservationMilestoneStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackReservationMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback reservation milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.limit as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackReservationMilestone {
        FallbackReservationMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            quantity: self.quantity,
            source: self.source.clone(),
            locale: self.locale.clone(),
            score: self.score,
            country: self.country.clone(),
            cipher: self.cipher.clone(),
            host: self.host.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a fallback reservation milestone in status draft.
pub fn new_fallback_reservation_milestone(code: &str, title: &str) -> FallbackReservationMilestone {
    FallbackReservationMilestone {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        quantity: 0,
        source: "".to_string(),
        locale: "".to_string(),
        score: 0.0,
        country: "".to_string(),
        cipher: "".to_string(),
        host: "".to_string(),
        limit: 0,
        status: FallbackReservationMilestoneStatus::Draft,
    }
}
