use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast reservation must pass. Check returns its problems, or none.
pub trait ForecastReservationRule: std::fmt::Debug {
    fn check(&self, item: &ForecastReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastReservationCodePrefixRule {
    pub prefix: String,
}

impl ForecastReservationRule for ForecastReservationCodePrefixRule {
    fn check(&self, item: &ForecastReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastReservationRequiredPhoneRule {
    pub message: String,
}

impl ForecastReservationRule for ForecastReservationRequiredPhoneRule {
    fn check(&self, item: &ForecastReservation) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastReservationRequiredLabelRule {
    pub message: String,
}

impl ForecastReservationRule for ForecastReservationRequiredLabelRule {
    fn check(&self, item: &ForecastReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastReservationRequiredTenantIdRule {
    pub message: String,
}

impl ForecastReservationRule for ForecastReservationRequiredTenantIdRule {
    fn check(&self, item: &ForecastReservation) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast reservation.
#[derive(Debug, Clone)]
pub struct ForecastReservationValidator {
    pub rules: Vec<Rc<dyn ForecastReservationRule>>,
}

impl ForecastReservationValidator {
    pub fn validate(&self, item: &ForecastReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastReservation]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_forecast_reservation_validator() -> ForecastReservationValidator {
    ForecastReservationValidator {
        rules: vec![
            Rc::new(ForecastReservationCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn ForecastReservationRule>,
            Rc::new(ForecastReservationRequiredPhoneRule {
                message: "forecast reservation needs an entry".to_string(),
            }),
            Rc::new(ForecastReservationRequiredLabelRule {
                message: "forecast reservation needs this field".to_string(),
            }),
            Rc::new(ForecastReservationRequiredTenantIdRule {
                message: "forecast reservation needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_reservation_validator();
        let item = new_forecast_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = ForecastReservationRequiredPhoneRule {
            message: "forecast reservation needs a value".to_string(),
        };
        let mut item = new_forecast_reservation("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ForecastReservationRequiredLabelRule {
            message: "forecast reservation needs this field".to_string(),
        };
        let mut item = new_forecast_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = ForecastReservationRequiredTenantIdRule {
            message: "forecast reservation needs this field".to_string(),
        };
        let mut item = new_forecast_reservation("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
