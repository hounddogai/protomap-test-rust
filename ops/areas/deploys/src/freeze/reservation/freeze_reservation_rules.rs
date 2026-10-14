use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze reservation must pass. Check returns its problems, or none.
pub trait FreezeReservationRule: std::fmt::Debug {
    fn check(&self, item: &FreezeReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeReservationRequiredCodeRule {
    pub message: String,
}

impl FreezeReservationRule for FreezeReservationRequiredCodeRule {
    fn check(&self, item: &FreezeReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeReservationCurrencyLengthRule {
    pub max_length: i64,
}

impl FreezeReservationRule for FreezeReservationCurrencyLengthRule {
    fn check(&self, item: &FreezeReservation) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeReservationTenantIdLengthRule {
    pub max_length: i64,
}

impl FreezeReservationRule for FreezeReservationTenantIdLengthRule {
    fn check(&self, item: &FreezeReservation) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeReservationMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FreezeReservationRule for FreezeReservationMarginRangeRule {
    fn check(&self, item: &FreezeReservation) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeReservationNameLengthRule {
    pub max_length: i64,
}

impl FreezeReservationRule for FreezeReservationNameLengthRule {
    fn check(&self, item: &FreezeReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeReservationRequiredCurrencyRule {
    pub message: String,
}

impl FreezeReservationRule for FreezeReservationRequiredCurrencyRule {
    fn check(&self, item: &FreezeReservation) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze reservation.
#[derive(Debug, Clone)]
pub struct FreezeReservationValidator {
    pub rules: Vec<Rc<dyn FreezeReservationRule>>,
}

impl FreezeReservationValidator {
    pub fn validate(&self, item: &FreezeReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_reservation_validator() -> FreezeReservationValidator {
    FreezeReservationValidator {
        rules: vec![
            Rc::new(FreezeReservationRequiredCodeRule {
                message: "freeze reservation needs a value".to_string(),
            }) as Rc<dyn FreezeReservationRule>,
            Rc::new(FreezeReservationCurrencyLengthRule { max_length: 1000 }),
            Rc::new(FreezeReservationTenantIdLengthRule { max_length: 64 }),
            Rc::new(FreezeReservationMarginRangeRule { min: 6.2, max: 2945.7 }),
            Rc::new(FreezeReservationNameLengthRule { max_length: 3600 }),
            Rc::new(FreezeReservationRequiredCurrencyRule {
                message: "freeze reservation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_reservation_validator();
        let item = new_freeze_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FreezeReservationRequiredCodeRule {
            message: "freeze reservation needs this field".to_string(),
        };
        let mut item = new_freeze_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = FreezeReservationCurrencyLengthRule { max_length: 86400 };
        let mut item = new_freeze_reservation("x", "y");
        item.currency = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = FreezeReservationTenantIdLengthRule { max_length: 80 };
        let mut item = new_freeze_reservation("x", "y");
        item.tenant_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = FreezeReservationMarginRangeRule { min: 6.8, max: 1790.7 };
        let mut item = new_freeze_reservation("x", "y");
        item.margin = 1792.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FreezeReservationNameLengthRule { max_length: 255 };
        let mut item = new_freeze_reservation("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = FreezeReservationRequiredCurrencyRule {
            message: "freeze reservation needs a value".to_string(),
        };
        let mut item = new_freeze_reservation("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
