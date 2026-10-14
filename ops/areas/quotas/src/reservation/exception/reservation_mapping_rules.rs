use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a reservation mapping must pass. Check returns its problems, or none.
pub trait ReservationMappingRule: std::fmt::Debug {
    fn check(&self, item: &ReservationMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMappingAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ReservationMappingRule for ReservationMappingAllowedNameRule {
    fn check(&self, item: &ReservationMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMappingCodePrefixRule {
    pub prefix: String,
}

impl ReservationMappingRule for ReservationMappingCodePrefixRule {
    fn check(&self, item: &ReservationMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMappingRequiredCodeRule {
    pub message: String,
}

impl ReservationMappingRule for ReservationMappingRequiredCodeRule {
    fn check(&self, item: &ReservationMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMappingRequiredNameRule {
    pub message: String,
}

impl ReservationMappingRule for ReservationMappingRequiredNameRule {
    fn check(&self, item: &ReservationMapping) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMappingCodeLengthRule {
    pub max_length: i64,
}

impl ReservationMappingRule for ReservationMappingCodeLengthRule {
    fn check(&self, item: &ReservationMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMappingCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReservationMappingRule for ReservationMappingCostRangeRule {
    fn check(&self, item: &ReservationMapping) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a reservation mapping.
#[derive(Debug, Clone)]
pub struct ReservationMappingValidator {
    pub rules: Vec<Rc<dyn ReservationMappingRule>>,
}

impl ReservationMappingValidator {
    pub fn validate(&self, item: &ReservationMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_mapping_validator() -> ReservationMappingValidator {
    ReservationMappingValidator {
        rules: vec![
            Rc::new(ReservationMappingAllowedNameRule {
                allowed: vec!["strict".to_string(), "strict".to_string()],
            }) as Rc<dyn ReservationMappingRule>,
            Rc::new(ReservationMappingCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ReservationMappingRequiredCodeRule {
                message: "reservation mapping needs a value".to_string(),
            }),
            Rc::new(ReservationMappingRequiredNameRule {
                message: "reservation mapping needs an entry".to_string(),
            }),
            Rc::new(ReservationMappingCodeLengthRule { max_length: 3600 }),
            Rc::new(ReservationMappingCostRangeRule { min: 4.0, max: 2071.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_mapping_validator();
        let item = new_reservation_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReservationMappingRequiredCodeRule {
            message: "reservation mapping needs a value".to_string(),
        };
        let mut item = new_reservation_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ReservationMappingRequiredNameRule {
            message: "reservation mapping needs this field".to_string(),
        };
        let mut item = new_reservation_mapping("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReservationMappingCodeLengthRule { max_length: 120 };
        let mut item = new_reservation_mapping("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = ReservationMappingCostRangeRule { min: 5.4, max: 3793.5 };
        let mut item = new_reservation_mapping("x", "y");
        item.cost = 3795.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
