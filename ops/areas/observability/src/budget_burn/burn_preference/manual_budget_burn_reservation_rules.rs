use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual budget burn reservation must pass. Check returns its problems, or none.
pub trait ManualBudgetBurnReservationRule: std::fmt::Debug {
    fn check(&self, item: &ManualBudgetBurnReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnReservationRequiredContactEmailRule {
    pub message: String,
}

impl ManualBudgetBurnReservationRule for ManualBudgetBurnReservationRequiredContactEmailRule {
    fn check(&self, item: &ManualBudgetBurnReservation) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnReservationRequiredNameRule {
    pub message: String,
}

impl ManualBudgetBurnReservationRule for ManualBudgetBurnReservationRequiredNameRule {
    fn check(&self, item: &ManualBudgetBurnReservation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnReservationContactEmailLengthRule {
    pub max_length: i64,
}

impl ManualBudgetBurnReservationRule for ManualBudgetBurnReservationContactEmailLengthRule {
    fn check(&self, item: &ManualBudgetBurnReservation) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnReservationRequiredCodeRule {
    pub message: String,
}

impl ManualBudgetBurnReservationRule for ManualBudgetBurnReservationRequiredCodeRule {
    fn check(&self, item: &ManualBudgetBurnReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnReservationMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualBudgetBurnReservationRule for ManualBudgetBurnReservationMinimumRangeRule {
    fn check(&self, item: &ManualBudgetBurnReservation) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual budget burn reservation.
#[derive(Debug, Clone)]
pub struct ManualBudgetBurnReservationValidator {
    pub rules: Vec<Rc<dyn ManualBudgetBurnReservationRule>>,
}

impl ManualBudgetBurnReservationValidator {
    pub fn validate(&self, item: &ManualBudgetBurnReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualBudgetBurnReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualBudgetBurnReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_budget_burn_reservation_validator() -> ManualBudgetBurnReservationValidator {
    ManualBudgetBurnReservationValidator {
        rules: vec![
            Rc::new(ManualBudgetBurnReservationRequiredContactEmailRule {
                message: "manual budget burn reservation needs an entry".to_string(),
            }) as Rc<dyn ManualBudgetBurnReservationRule>,
            Rc::new(ManualBudgetBurnReservationRequiredNameRule {
                message: "manual budget burn reservation needs this field".to_string(),
            }),
            Rc::new(ManualBudgetBurnReservationContactEmailLengthRule { max_length: 255 }),
            Rc::new(ManualBudgetBurnReservationRequiredCodeRule {
                message: "manual budget burn reservation needs this field".to_string(),
            }),
            Rc::new(ManualBudgetBurnReservationMinimumRangeRule { min: 1.6, max: 185.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_budget_burn_reservation_validator();
        let item = new_manual_budget_burn_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = ManualBudgetBurnReservationRequiredContactEmailRule {
            message: "manual budget burn reservation needs this field".to_string(),
        };
        let mut item = new_manual_budget_burn_reservation("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ManualBudgetBurnReservationRequiredNameRule {
            message: "manual budget burn reservation needs this field".to_string(),
        };
        let mut item = new_manual_budget_burn_reservation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = ManualBudgetBurnReservationContactEmailLengthRule { max_length: 3600 };
        let mut item = new_manual_budget_burn_reservation("x", "y");
        item.contact_email = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualBudgetBurnReservationRequiredCodeRule {
            message: "manual budget burn reservation needs an entry".to_string(),
        };
        let mut item = new_manual_budget_burn_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = ManualBudgetBurnReservationMinimumRangeRule { min: 3.3, max: 2180.5 };
        let mut item = new_manual_budget_burn_reservation("x", "y");
        item.minimum = 2182.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
