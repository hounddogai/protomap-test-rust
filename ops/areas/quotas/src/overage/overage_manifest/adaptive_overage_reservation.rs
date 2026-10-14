use super::*;

use acme_kit::text;

/// Where a adaptive overage reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveOverageReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AdaptiveOverageReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AdaptiveOverageReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive overage reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveOverageReservation {
    pub code: String,
    pub label: String,
    pub source: String,
    pub rotation_days: i64,
    pub window_days: i64,
    pub status: AdaptiveOverageReservationStatus,
}

impl AdaptiveOverageReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveOverageReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveOverageReservationStatus) -> bool {
        match self.status {
            AdaptiveOverageReservationStatus::Created => {
                return next == AdaptiveOverageReservationStatus::Validated
                    || next == AdaptiveOverageReservationStatus::Retired;
            }
            AdaptiveOverageReservationStatus::Validated => {
                return next == AdaptiveOverageReservationStatus::Retired;
            }
            AdaptiveOverageReservationStatus::Published => {
                return next == AdaptiveOverageReservationStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveOverageReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive overage reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.rotation_days as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveOverageReservation {
        AdaptiveOverageReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            rotation_days: self.rotation_days,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive overage reservation in status created.
pub fn new_adaptive_overage_reservation(code: &str, label: &str) -> AdaptiveOverageReservation {
    AdaptiveOverageReservation {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        rotation_days: 0,
        window_days: 0,
        status: AdaptiveOverageReservationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_adaptive_overage_reservation("code-975", "label-174");
        assert_eq!(item.status, AdaptiveOverageReservationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_adaptive_overage_reservation("code-975", "label-174");
        if item.move_to(AdaptiveOverageReservationStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveOverageReservationStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_adaptive_overage_reservation("code-975", "label-174");
        item.status = AdaptiveOverageReservationStatus::Retired;
        assert!(
            item.move_to(AdaptiveOverageReservationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
