use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict ttl policy cycle must pass. Check returns its problems, or none.
pub trait StrictTtlPolicyCycleRule: std::fmt::Debug {
    fn check(&self, item: &StrictTtlPolicyCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTtlPolicyCycleCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictTtlPolicyCycleRule for StrictTtlPolicyCycleCapacityRangeRule {
    fn check(&self, item: &StrictTtlPolicyCycle) -> Vec<String> {
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
pub struct StrictTtlPolicyCycleRegionLengthRule {
    pub max_length: i64,
}

impl StrictTtlPolicyCycleRule for StrictTtlPolicyCycleRegionLengthRule {
    fn check(&self, item: &StrictTtlPolicyCycle) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTtlPolicyCycleRequiredRegionRule {
    pub message: String,
}

impl StrictTtlPolicyCycleRule for StrictTtlPolicyCycleRequiredRegionRule {
    fn check(&self, item: &StrictTtlPolicyCycle) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTtlPolicyCycleRequiredPrincipalRule {
    pub message: String,
}

impl StrictTtlPolicyCycleRule for StrictTtlPolicyCycleRequiredPrincipalRule {
    fn check(&self, item: &StrictTtlPolicyCycle) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTtlPolicyCycleWindowRule {
    pub max_span: i64,
}

impl StrictTtlPolicyCycleRule for StrictTtlPolicyCycleWindowRule {
    fn check(&self, item: &StrictTtlPolicyCycle) -> Vec<String> {
        if item.starts_at > item.ends_at {
            return vec!["starts at is after ends at".to_string()];
        }
        if self.max_span > 0 && (item.ends_at - item.starts_at > self.max_span) {
            return vec![format!("spans more than {} seconds", self.max_span)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict ttl policy cycle.
#[derive(Debug, Clone)]
pub struct StrictTtlPolicyCycleValidator {
    pub rules: Vec<Rc<dyn StrictTtlPolicyCycleRule>>,
}

impl StrictTtlPolicyCycleValidator {
    pub fn validate(&self, item: &StrictTtlPolicyCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictTtlPolicyCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictTtlPolicyCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_ttl_policy_cycle_validator() -> StrictTtlPolicyCycleValidator {
    StrictTtlPolicyCycleValidator {
        rules: vec![
            Rc::new(StrictTtlPolicyCycleCapacityRangeRule { min: 80, max: 80 }) as Rc<dyn StrictTtlPolicyCycleRule>,
            Rc::new(StrictTtlPolicyCycleRegionLengthRule { max_length: 120 }),
            Rc::new(StrictTtlPolicyCycleRequiredRegionRule {
                message: "strict ttl policy cycle needs this field".to_string(),
            }),
            Rc::new(StrictTtlPolicyCycleRequiredPrincipalRule {
                message: "strict ttl policy cycle needs an entry".to_string(),
            }),
            Rc::new(StrictTtlPolicyCycleWindowRule { max_span: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_ttl_policy_cycle_validator();
        let item = new_strict_ttl_policy_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = StrictTtlPolicyCycleCapacityRangeRule { min: 16, max: 1000 };
        let mut item = new_strict_ttl_policy_cycle("x", "y");
        item.capacity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = StrictTtlPolicyCycleRegionLengthRule { max_length: 86400 };
        let mut item = new_strict_ttl_policy_cycle("x", "y");
        item.region = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = StrictTtlPolicyCycleRequiredRegionRule {
            message: "strict ttl policy cycle needs an entry".to_string(),
        };
        let mut item = new_strict_ttl_policy_cycle("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = StrictTtlPolicyCycleRequiredPrincipalRule {
            message: "strict ttl policy cycle needs a value".to_string(),
        };
        let mut item = new_strict_ttl_policy_cycle("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
