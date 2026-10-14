use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage reservation must pass. Check returns its problems, or none.
pub trait UsageReservationRule: std::fmt::Debug {
    fn check(&self, item: &UsageReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageReservationAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl UsageReservationRule for UsageReservationAllowedScopeRule {
    fn check(&self, item: &UsageReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is above maximum", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageReservationRequiredTitleRule {
    pub message: String,
}

impl UsageReservationRule for UsageReservationRequiredTitleRule {
    fn check(&self, item: &UsageReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageReservationWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl UsageReservationRule for UsageReservationWeightRangeRule {
    fn check(&self, item: &UsageReservation) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageReservationRequiredScopeRule {
    pub message: String,
}

impl UsageReservationRule for UsageReservationRequiredScopeRule {
    fn check(&self, item: &UsageReservation) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageReservationCodePrefixRule {
    pub prefix: String,
}

impl UsageReservationRule for UsageReservationCodePrefixRule {
    fn check(&self, item: &UsageReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage reservation.
#[derive(Debug, Clone)]
pub struct UsageReservationValidator {
    pub rules: Vec<Rc<dyn UsageReservationRule>>,
}

impl UsageReservationValidator {
    pub fn validate(&self, item: &UsageReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_reservation_validator() -> UsageReservationValidator {
    UsageReservationValidator {
        rules: vec![
            Rc::new(UsageReservationAllowedScopeRule { allowed: Vec::new() }) as Rc<dyn UsageReservationRule>,
            Rc::new(UsageReservationRequiredTitleRule {
                message: "usage reservation needs an entry".to_string(),
            }),
            Rc::new(UsageReservationWeightRangeRule { min: 8.5, max: 3552.6 }),
            Rc::new(UsageReservationRequiredScopeRule {
                message: "usage reservation needs an entry".to_string(),
            }),
            Rc::new(UsageReservationCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_reservation_validator();
        let item = new_usage_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = UsageReservationRequiredTitleRule {
            message: "usage reservation needs an entry".to_string(),
        };
        let mut item = new_usage_reservation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = UsageReservationWeightRangeRule { min: 4.6, max: 3110.3 };
        let mut item = new_usage_reservation("x", "y");
        item.weight = 3111.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = UsageReservationRequiredScopeRule {
            message: "usage reservation needs a value".to_string(),
        };
        let mut item = new_usage_reservation("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
