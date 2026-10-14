use super::*;

use acme_kit::text;

/// Where a lenient rule reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRuleReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientRuleReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientRuleReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient rule reservation as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleReservation {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub window_days: i64,
    pub maximum: f64,
    pub quantity: i64,
    pub status: LenientRuleReservationStatus,
}

impl LenientRuleReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRuleReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRuleReservationStatus) -> bool {
        match self.status {
            LenientRuleReservationStatus::Requested => {
                return next == LenientRuleReservationStatus::Granted;
            }
            LenientRuleReservationStatus::Granted => {
                match next {
                    LenientRuleReservationStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRuleReservationStatus::Revoked => {
                return next == LenientRuleReservationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientRuleReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient rule reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient rule reservation", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.maximum - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRuleReservation {
        LenientRuleReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            window_days: self.window_days,
            maximum: self.maximum,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient rule reservation in status requested.
pub fn new_lenient_rule_reservation(code: &str, title: &str) -> LenientRuleReservation {
    LenientRuleReservation {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        window_days: 0,
        maximum: 0.0,
        quantity: 0,
        status: LenientRuleReservationStatus::Requested,
    }
}
