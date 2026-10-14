use super::*;

use acme_kit::text;

/// Where a strict forecast cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictForecastCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl StrictForecastCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for StrictForecastCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict forecast cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastCycle {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub currency: String,
    pub notes: String,
    pub status: StrictForecastCycleStatus,
}

impl StrictForecastCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictForecastCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictForecastCycleStatus) -> bool {
        match self.status {
            StrictForecastCycleStatus::Proposed => {
                return next == StrictForecastCycleStatus::Approved || next == StrictForecastCycleStatus::Withdrawn;
            }
            StrictForecastCycleStatus::Approved => {
                match next {
                    StrictForecastCycleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictForecastCycleStatus::Rejected => {
                match next {
                    StrictForecastCycleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictForecastCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict forecast cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> StrictForecastCycle {
        StrictForecastCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            currency: self.currency.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict forecast cycle in status proposed.
pub fn new_strict_forecast_cycle(code: &str, label: &str) -> StrictForecastCycle {
    StrictForecastCycle {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        currency: "".to_string(),
        notes: "".to_string(),
        status: StrictForecastCycleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_strict_forecast_cycle("code-661", "label-941");
        assert_eq!(item.status, StrictForecastCycleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_strict_forecast_cycle("code-661", "label-941");
        if item.move_to(StrictForecastCycleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictForecastCycleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_strict_forecast_cycle("code-661", "label-941");
        item.status = StrictForecastCycleStatus::Withdrawn;
        assert!(
            item.move_to(StrictForecastCycleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
