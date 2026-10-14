use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending canary mapping must pass. Check returns its problems, or none.
pub trait PendingCanaryMappingRule: std::fmt::Debug {
    fn check(&self, item: &PendingCanaryMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryMappingCodeLengthRule {
    pub max_length: i64,
}

impl PendingCanaryMappingRule for PendingCanaryMappingCodeLengthRule {
    fn check(&self, item: &PendingCanaryMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryMappingRequiredCodeRule {
    pub message: String,
}

impl PendingCanaryMappingRule for PendingCanaryMappingRequiredCodeRule {
    fn check(&self, item: &PendingCanaryMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryMappingTitleLengthRule {
    pub max_length: i64,
}

impl PendingCanaryMappingRule for PendingCanaryMappingTitleLengthRule {
    fn check(&self, item: &PendingCanaryMapping) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryMappingCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingCanaryMappingRule for PendingCanaryMappingCapacityRangeRule {
    fn check(&self, item: &PendingCanaryMapping) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryMappingCountryLengthRule {
    pub max_length: i64,
}

impl PendingCanaryMappingRule for PendingCanaryMappingCountryLengthRule {
    fn check(&self, item: &PendingCanaryMapping) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryMappingStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingCanaryMappingRule for PendingCanaryMappingStartsAtRangeRule {
    fn check(&self, item: &PendingCanaryMapping) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending canary mapping.
#[derive(Debug, Clone)]
pub struct PendingCanaryMappingValidator {
    pub rules: Vec<Rc<dyn PendingCanaryMappingRule>>,
}

impl PendingCanaryMappingValidator {
    pub fn validate(&self, item: &PendingCanaryMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingCanaryMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingCanaryMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_canary_mapping_validator() -> PendingCanaryMappingValidator {
    PendingCanaryMappingValidator {
        rules: vec![
            Rc::new(PendingCanaryMappingCodeLengthRule { max_length: 16 }) as Rc<dyn PendingCanaryMappingRule>,
            Rc::new(PendingCanaryMappingRequiredCodeRule {
                message: "pending canary mapping needs this field".to_string(),
            }),
            Rc::new(PendingCanaryMappingTitleLengthRule { max_length: 32 }),
            Rc::new(PendingCanaryMappingCapacityRangeRule { min: 32, max: 1000 }),
            Rc::new(PendingCanaryMappingCountryLengthRule { max_length: 16 }),
            Rc::new(PendingCanaryMappingStartsAtRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_canary_mapping_validator();
        let item = new_pending_canary_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingCanaryMappingCodeLengthRule { max_length: 16 };
        let mut item = new_pending_canary_mapping("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingCanaryMappingRequiredCodeRule {
            message: "pending canary mapping needs this field".to_string(),
        };
        let mut item = new_pending_canary_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingCanaryMappingTitleLengthRule { max_length: 64 };
        let mut item = new_pending_canary_mapping("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = PendingCanaryMappingCapacityRangeRule { min: 32, max: 80 };
        let mut item = new_pending_canary_mapping("x", "y");
        item.capacity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = PendingCanaryMappingCountryLengthRule { max_length: 3600 };
        let mut item = new_pending_canary_mapping("x", "y");
        item.country = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = PendingCanaryMappingStartsAtRangeRule { min: 64, max: 120 };
        let mut item = new_pending_canary_mapping("x", "y");
        item.starts_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
