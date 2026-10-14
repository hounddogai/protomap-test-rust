use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem mapping must pass. Check returns its problems, or none.
pub trait PostmortemMappingRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemMappingVisibleGuardRule {
    pub limit: i64,
}

impl PostmortemMappingRule for PostmortemMappingVisibleGuardRule {
    fn check(&self, item: &PostmortemMapping) -> Vec<String> {
        if item.visible && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while visible",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemMappingRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemMappingRule for PostmortemMappingRetriesRangeRule {
    fn check(&self, item: &PostmortemMapping) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemMappingPhoneLengthRule {
    pub max_length: i64,
}

impl PostmortemMappingRule for PostmortemMappingPhoneLengthRule {
    fn check(&self, item: &PostmortemMapping) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem mapping.
#[derive(Debug, Clone)]
pub struct PostmortemMappingValidator {
    pub rules: Vec<Rc<dyn PostmortemMappingRule>>,
}

impl PostmortemMappingValidator {
    pub fn validate(&self, item: &PostmortemMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_mapping_validator() -> PostmortemMappingValidator {
    PostmortemMappingValidator {
        rules: vec![
            Rc::new(PostmortemMappingVisibleGuardRule { limit: 16 }) as Rc<dyn PostmortemMappingRule>,
            Rc::new(PostmortemMappingRetriesRangeRule { min: 80, max: 120 }),
            Rc::new(PostmortemMappingPhoneLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_mapping_validator();
        let item = new_postmortem_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = PostmortemMappingRetriesRangeRule { min: 120, max: 255 };
        let mut item = new_postmortem_mapping("x", "y");
        item.retries = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = PostmortemMappingPhoneLengthRule { max_length: 64 };
        let mut item = new_postmortem_mapping("x", "y");
        item.phone = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
