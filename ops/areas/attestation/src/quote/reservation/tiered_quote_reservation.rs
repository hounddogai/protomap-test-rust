use super::*;

use acme_kit::text;

/// Where a tiered quote reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredQuoteReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredQuoteReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TieredQuoteReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered quote reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteReservation {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub cipher: String,
    pub currency: String,
    pub visible: bool,
    pub source: String,
    pub ceiling: f64,
    pub window_days: i64,
    pub sequence: i64,
    pub status: TieredQuoteReservationStatus,
}

impl TieredQuoteReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredQuoteReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredQuoteReservationStatus) -> bool {
        match self.status {
            TieredQuoteReservationStatus::Planned => {
                match next {
                    TieredQuoteReservationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredQuoteReservationStatus::Scheduled => {
                return next == TieredQuoteReservationStatus::Cancelled;
            }
            TieredQuoteReservationStatus::InProgress => {
                return next == TieredQuoteReservationStatus::Completed;
            }
            TieredQuoteReservationStatus::Completed => {
                return next == TieredQuoteReservationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredQuoteReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered quote reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered quote reservation", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.ceiling + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredQuoteReservation {
        TieredQuoteReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            cipher: self.cipher.clone(),
            currency: self.currency.clone(),
            visible: self.visible,
            source: self.source.clone(),
            ceiling: self.ceiling,
            window_days: self.window_days,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a tiered quote reservation in status planned.
pub fn new_tiered_quote_reservation(code: &str, name: &str) -> TieredQuoteReservation {
    TieredQuoteReservation {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        cipher: "".to_string(),
        currency: "".to_string(),
        visible: false,
        source: "".to_string(),
        ceiling: 0.0,
        window_days: 0,
        sequence: 0,
        status: TieredQuoteReservationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_quote_reservation("code-405", "name-525");
        assert_eq!(item.status, TieredQuoteReservationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_tiered_quote_reservation("code-405", "name-525");
        if item.move_to(TieredQuoteReservationStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredQuoteReservationStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_quote_reservation("code-405", "name-525");
        item.status = TieredQuoteReservationStatus::Cancelled;
        assert!(
            item.move_to(TieredQuoteReservationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
