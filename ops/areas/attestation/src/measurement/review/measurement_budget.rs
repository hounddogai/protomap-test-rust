use super::*;

use acme_kit::text;

/// Where a measurement budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementBudgetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl MeasurementBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for MeasurementBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a measurement budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBudget {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub description: String,
    pub principal: String,
    pub amount: f64,
    pub external_ref: String,
    pub grace_days: i64,
    pub rate: f64,
    pub timezone: String,
    pub status: MeasurementBudgetStatus,
}

impl MeasurementBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementBudgetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementBudgetStatus) -> bool {
        match self.status {
            MeasurementBudgetStatus::Requested => {
                match next {
                    MeasurementBudgetStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementBudgetStatus::Granted => {
                match next {
                    MeasurementBudgetStatus::Revoked | MeasurementBudgetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementBudgetStatus::Revoked => {
                return next == MeasurementBudgetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.budget - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementBudget {
        MeasurementBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            description: self.description.clone(),
            principal: self.principal.clone(),
            amount: self.amount,
            external_ref: self.external_ref.clone(),
            grace_days: self.grace_days,
            rate: self.rate,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a measurement budget in status requested.
pub fn new_measurement_budget(code: &str, name: &str) -> MeasurementBudget {
    MeasurementBudget {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        description: "".to_string(),
        principal: "".to_string(),
        amount: 0.0,
        external_ref: "".to_string(),
        grace_days: 0,
        rate: 0.0,
        timezone: "".to_string(),
        status: MeasurementBudgetStatus::Requested,
    }
}
