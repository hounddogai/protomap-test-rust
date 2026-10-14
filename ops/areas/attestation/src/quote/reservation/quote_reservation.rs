use super::*;

use acme_kit::text;

/// Where a quote reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl QuoteReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for QuoteReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a quote reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteReservation {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub rate: f64,
    pub burst: i64,
    pub timezone: String,
    pub amount: f64,
    pub active: bool,
    pub status: QuoteReservationStatus,
}

impl QuoteReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteReservationStatus) -> bool {
        match self.status {
            QuoteReservationStatus::Created => {
                return next == QuoteReservationStatus::Validated || next == QuoteReservationStatus::Retired;
            }
            QuoteReservationStatus::Validated => {
                match next {
                    QuoteReservationStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteReservationStatus::Published => {
                return next == QuoteReservationStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "quote reservation", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.rate + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteReservation {
        QuoteReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            rate: self.rate,
            burst: self.burst,
            timezone: self.timezone.clone(),
            amount: self.amount,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a quote reservation in status created.
pub fn new_quote_reservation(code: &str, title: &str) -> QuoteReservation {
    QuoteReservation {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        rate: 0.0,
        burst: 0,
        timezone: "".to_string(),
        amount: 0.0,
        active: false,
        status: QuoteReservationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_quote_reservation("code-142", "title-903");
        assert_eq!(item.status, QuoteReservationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_quote_reservation("code-142", "title-903");
        if item.move_to(QuoteReservationStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteReservationStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_quote_reservation("code-142", "title-903");
        item.status = QuoteReservationStatus::Retired;
        assert!(
            item.move_to(QuoteReservationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
