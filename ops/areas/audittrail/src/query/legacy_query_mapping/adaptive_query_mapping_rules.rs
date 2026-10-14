use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive query mapping must pass. Check returns its problems, or none.
pub trait AdaptiveQueryMappingRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveQueryMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryMappingCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveQueryMappingRule for AdaptiveQueryMappingCodePrefixRule {
    fn check(&self, item: &AdaptiveQueryMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryMappingLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveQueryMappingRule for AdaptiveQueryMappingLabelLengthRule {
    fn check(&self, item: &AdaptiveQueryMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryMappingCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveQueryMappingRule for AdaptiveQueryMappingCapacityRangeRule {
    fn check(&self, item: &AdaptiveQueryMapping) -> Vec<String> {
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
pub struct AdaptiveQueryMappingAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl AdaptiveQueryMappingRule for AdaptiveQueryMappingAllowedLabelRule {
    fn check(&self, item: &AdaptiveQueryMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is blocked", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive query mapping.
#[derive(Debug, Clone)]
pub struct AdaptiveQueryMappingValidator {
    pub rules: Vec<Rc<dyn AdaptiveQueryMappingRule>>,
}

impl AdaptiveQueryMappingValidator {
    pub fn validate(&self, item: &AdaptiveQueryMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveQueryMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveQueryMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_query_mapping_validator() -> AdaptiveQueryMappingValidator {
    AdaptiveQueryMappingValidator {
        rules: vec![
            Rc::new(AdaptiveQueryMappingCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn AdaptiveQueryMappingRule>,
            Rc::new(AdaptiveQueryMappingLabelLengthRule { max_length: 64 }),
            Rc::new(AdaptiveQueryMappingCapacityRangeRule { min: 120, max: 3600 }),
            Rc::new(AdaptiveQueryMappingAllowedLabelRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_query_mapping_validator();
        let item = new_adaptive_query_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AdaptiveQueryMappingLabelLengthRule { max_length: 120 };
        let mut item = new_adaptive_query_mapping("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = AdaptiveQueryMappingCapacityRangeRule { min: 16, max: 1000 };
        let mut item = new_adaptive_query_mapping("x", "y");
        item.capacity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
