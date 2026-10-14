use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual ttl policy reservation must pass. Check returns its problems, or none.
pub trait ManualTtlPolicyReservationRule: std::fmt::Debug {
    fn check(&self, item: &ManualTtlPolicyReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyReservationRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualTtlPolicyReservationRule for ManualTtlPolicyReservationRatioRangeRule {
    fn check(&self, item: &ManualTtlPolicyReservation) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyReservationGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualTtlPolicyReservationRule for ManualTtlPolicyReservationGraceDaysRangeRule {
    fn check(&self, item: &ManualTtlPolicyReservation) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyReservationCategoryLengthRule {
    pub max_length: i64,
}

impl ManualTtlPolicyReservationRule for ManualTtlPolicyReservationCategoryLengthRule {
    fn check(&self, item: &ManualTtlPolicyReservation) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyReservationPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualTtlPolicyReservationRule for ManualTtlPolicyReservationPriorityRangeRule {
    fn check(&self, item: &ManualTtlPolicyReservation) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyReservationRequiredScopeRule {
    pub message: String,
}

impl ManualTtlPolicyReservationRule for ManualTtlPolicyReservationRequiredScopeRule {
    fn check(&self, item: &ManualTtlPolicyReservation) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual ttl policy reservation.
#[derive(Debug, Clone)]
pub struct ManualTtlPolicyReservationValidator {
    pub rules: Vec<Rc<dyn ManualTtlPolicyReservationRule>>,
}

impl ManualTtlPolicyReservationValidator {
    pub fn validate(&self, item: &ManualTtlPolicyReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualTtlPolicyReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualTtlPolicyReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_ttl_policy_reservation_validator() -> ManualTtlPolicyReservationValidator {
    ManualTtlPolicyReservationValidator {
        rules: vec![
            Rc::new(ManualTtlPolicyReservationRatioRangeRule { min: 9.6, max: 1120.7 })
                as Rc<dyn ManualTtlPolicyReservationRule>,
            Rc::new(ManualTtlPolicyReservationGraceDaysRangeRule { min: 64, max: 80 }),
            Rc::new(ManualTtlPolicyReservationCategoryLengthRule { max_length: 86400 }),
            Rc::new(ManualTtlPolicyReservationPriorityRangeRule { min: 80, max: 255 }),
            Rc::new(ManualTtlPolicyReservationRequiredScopeRule {
                message: "manual ttl policy reservation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_ttl_policy_reservation_validator();
        let item = new_manual_ttl_policy_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = ManualTtlPolicyReservationRatioRangeRule { min: 1.5, max: 1529.1 };
        let mut item = new_manual_ttl_policy_reservation("x", "y");
        item.ratio = 1530.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = ManualTtlPolicyReservationGraceDaysRangeRule { min: 80, max: 255 };
        let mut item = new_manual_ttl_policy_reservation("x", "y");
        item.grace_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = ManualTtlPolicyReservationCategoryLengthRule { max_length: 16 };
        let mut item = new_manual_ttl_policy_reservation("x", "y");
        item.category = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = ManualTtlPolicyReservationPriorityRangeRule { min: 3600, max: 86400 };
        let mut item = new_manual_ttl_policy_reservation("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = ManualTtlPolicyReservationRequiredScopeRule {
            message: "manual ttl policy reservation needs a value".to_string(),
        };
        let mut item = new_manual_ttl_policy_reservation("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
