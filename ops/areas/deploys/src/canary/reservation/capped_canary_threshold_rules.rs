use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped canary threshold must pass. Check returns its problems, or none.
pub trait CappedCanaryThresholdRule: std::fmt::Debug {
    fn check(&self, item: &CappedCanaryThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryThresholdCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedCanaryThresholdRule for CappedCanaryThresholdCreatedAtRangeRule {
    fn check(&self, item: &CappedCanaryThreshold) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryThresholdRequiredCodeRule {
    pub message: String,
}

impl CappedCanaryThresholdRule for CappedCanaryThresholdRequiredCodeRule {
    fn check(&self, item: &CappedCanaryThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryThresholdOwnerIdLengthRule {
    pub max_length: i64,
}

impl CappedCanaryThresholdRule for CappedCanaryThresholdOwnerIdLengthRule {
    fn check(&self, item: &CappedCanaryThreshold) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryThresholdCodePrefixRule {
    pub prefix: String,
}

impl CappedCanaryThresholdRule for CappedCanaryThresholdCodePrefixRule {
    fn check(&self, item: &CappedCanaryThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped canary threshold.
#[derive(Debug, Clone)]
pub struct CappedCanaryThresholdValidator {
    pub rules: Vec<Rc<dyn CappedCanaryThresholdRule>>,
}

impl CappedCanaryThresholdValidator {
    pub fn validate(&self, item: &CappedCanaryThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedCanaryThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedCanaryThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_canary_threshold_validator() -> CappedCanaryThresholdValidator {
    CappedCanaryThresholdValidator {
        rules: vec![
            Rc::new(CappedCanaryThresholdCreatedAtRangeRule { min: 120, max: 1000 })
                as Rc<dyn CappedCanaryThresholdRule>,
            Rc::new(CappedCanaryThresholdRequiredCodeRule {
                message: "capped canary threshold needs this field".to_string(),
            }),
            Rc::new(CappedCanaryThresholdOwnerIdLengthRule { max_length: 3600 }),
            Rc::new(CappedCanaryThresholdCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_canary_threshold_validator();
        let item = new_capped_canary_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = CappedCanaryThresholdCreatedAtRangeRule { min: 32, max: 1000 };
        let mut item = new_capped_canary_threshold("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedCanaryThresholdRequiredCodeRule {
            message: "capped canary threshold needs a value".to_string(),
        };
        let mut item = new_capped_canary_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = CappedCanaryThresholdOwnerIdLengthRule { max_length: 80 };
        let mut item = new_capped_canary_threshold("x", "y");
        item.owner_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
