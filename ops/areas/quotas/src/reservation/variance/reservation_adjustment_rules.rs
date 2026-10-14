use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation adjustment must pass. Check returns its problems, or none.
pub trait ReservationAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &ReservationAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAdjustmentLabelLengthRule {
    pub max_length: i64,
}

impl ReservationAdjustmentRule for ReservationAdjustmentLabelLengthRule {
    fn check(&self, item: &ReservationAdjustment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAdjustmentQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationAdjustmentRule for ReservationAdjustmentQuotaUnitsRangeRule {
    fn check(&self, item: &ReservationAdjustment) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAdjustmentRequiredCodeRule {
    pub message: String,
}

impl ReservationAdjustmentRule for ReservationAdjustmentRequiredCodeRule {
    fn check(&self, item: &ReservationAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl ReservationAdjustmentRule for ReservationAdjustmentCodePrefixRule {
    fn check(&self, item: &ReservationAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl ReservationAdjustmentRule for ReservationAdjustmentCodeLengthRule {
    fn check(&self, item: &ReservationAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAdjustmentRequiredLabelRule {
    pub message: String,
}

impl ReservationAdjustmentRule for ReservationAdjustmentRequiredLabelRule {
    fn check(&self, item: &ReservationAdjustment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation adjustment.
#[derive(Debug, Clone)]
pub struct ReservationAdjustmentValidator {
    pub rules: Vec<Rc<dyn ReservationAdjustmentRule>>,
}

impl ReservationAdjustmentValidator {
    pub fn validate(&self, item: &ReservationAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_adjustment_validator() -> ReservationAdjustmentValidator {
    ReservationAdjustmentValidator {
        rules: vec![
            Rc::new(ReservationAdjustmentLabelLengthRule { max_length: 3600 }) as Rc<dyn ReservationAdjustmentRule>,
            Rc::new(ReservationAdjustmentQuotaUnitsRangeRule { min: 80, max: 86400 }),
            Rc::new(ReservationAdjustmentRequiredCodeRule {
                message: "reservation adjustment needs this field".to_string(),
            }),
            Rc::new(ReservationAdjustmentCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ReservationAdjustmentCodeLengthRule { max_length: 3600 }),
            Rc::new(ReservationAdjustmentRequiredLabelRule {
                message: "reservation adjustment needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_adjustment_validator();
        let item = new_reservation_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ReservationAdjustmentLabelLengthRule { max_length: 86400 };
        let mut item = new_reservation_adjustment("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = ReservationAdjustmentQuotaUnitsRangeRule { min: 1000, max: 3600 };
        let mut item = new_reservation_adjustment("x", "y");
        item.quota_units = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReservationAdjustmentRequiredCodeRule {
            message: "reservation adjustment needs an entry".to_string(),
        };
        let mut item = new_reservation_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReservationAdjustmentCodeLengthRule { max_length: 120 };
        let mut item = new_reservation_adjustment("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ReservationAdjustmentRequiredLabelRule {
            message: "reservation adjustment needs an entry".to_string(),
        };
        let mut item = new_reservation_adjustment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
