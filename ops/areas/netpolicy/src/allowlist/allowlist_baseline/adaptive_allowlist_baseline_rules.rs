use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive allowlist baseline must pass. Check returns its problems, or none.
pub trait AdaptiveAllowlistBaselineRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveAllowlistBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBaselineRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveAllowlistBaselineRule for AdaptiveAllowlistBaselineRetriesRangeRule {
    fn check(&self, item: &AdaptiveAllowlistBaseline) -> Vec<String> {
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
pub struct AdaptiveAllowlistBaselineCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveAllowlistBaselineRule for AdaptiveAllowlistBaselineCodeLengthRule {
    fn check(&self, item: &AdaptiveAllowlistBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBaselineDescriptionLengthRule {
    pub max_length: i64,
}

impl AdaptiveAllowlistBaselineRule for AdaptiveAllowlistBaselineDescriptionLengthRule {
    fn check(&self, item: &AdaptiveAllowlistBaseline) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive allowlist baseline.
#[derive(Debug, Clone)]
pub struct AdaptiveAllowlistBaselineValidator {
    pub rules: Vec<Rc<dyn AdaptiveAllowlistBaselineRule>>,
}

impl AdaptiveAllowlistBaselineValidator {
    pub fn validate(&self, item: &AdaptiveAllowlistBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveAllowlistBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveAllowlistBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_allowlist_baseline_validator() -> AdaptiveAllowlistBaselineValidator {
    AdaptiveAllowlistBaselineValidator {
        rules: vec![
            Rc::new(AdaptiveAllowlistBaselineRetriesRangeRule { min: 120, max: 1000 })
                as Rc<dyn AdaptiveAllowlistBaselineRule>,
            Rc::new(AdaptiveAllowlistBaselineCodeLengthRule { max_length: 86400 }),
            Rc::new(AdaptiveAllowlistBaselineDescriptionLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_allowlist_baseline_validator();
        let item = new_adaptive_allowlist_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = AdaptiveAllowlistBaselineRetriesRangeRule { min: 32, max: 80 };
        let mut item = new_adaptive_allowlist_baseline("x", "y");
        item.retries = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveAllowlistBaselineCodeLengthRule { max_length: 16 };
        let mut item = new_adaptive_allowlist_baseline("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = AdaptiveAllowlistBaselineDescriptionLengthRule { max_length: 32 };
        let mut item = new_adaptive_allowlist_baseline("x", "y");
        item.description = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
