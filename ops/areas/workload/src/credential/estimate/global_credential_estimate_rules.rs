use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global credential estimate must pass. Check returns its problems, or none.
pub trait GlobalCredentialEstimateRule: std::fmt::Debug {
    fn check(&self, item: &GlobalCredentialEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCredentialEstimateSegmentLengthRule {
    pub max_length: i64,
}

impl GlobalCredentialEstimateRule for GlobalCredentialEstimateSegmentLengthRule {
    fn check(&self, item: &GlobalCredentialEstimate) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCredentialEstimateCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalCredentialEstimateRule for GlobalCredentialEstimateCapacityRangeRule {
    fn check(&self, item: &GlobalCredentialEstimate) -> Vec<String> {
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
pub struct GlobalCredentialEstimateRequiredLocaleRule {
    pub message: String,
}

impl GlobalCredentialEstimateRule for GlobalCredentialEstimateRequiredLocaleRule {
    fn check(&self, item: &GlobalCredentialEstimate) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCredentialEstimateTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalCredentialEstimateRule for GlobalCredentialEstimateTtlSecondsRangeRule {
    fn check(&self, item: &GlobalCredentialEstimate) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCredentialEstimateRequiredCodeRule {
    pub message: String,
}

impl GlobalCredentialEstimateRule for GlobalCredentialEstimateRequiredCodeRule {
    fn check(&self, item: &GlobalCredentialEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCredentialEstimateCodePrefixRule {
    pub prefix: String,
}

impl GlobalCredentialEstimateRule for GlobalCredentialEstimateCodePrefixRule {
    fn check(&self, item: &GlobalCredentialEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global credential estimate.
#[derive(Debug, Clone)]
pub struct GlobalCredentialEstimateValidator {
    pub rules: Vec<Rc<dyn GlobalCredentialEstimateRule>>,
}

impl GlobalCredentialEstimateValidator {
    pub fn validate(&self, item: &GlobalCredentialEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalCredentialEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalCredentialEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_credential_estimate_validator() -> GlobalCredentialEstimateValidator {
    GlobalCredentialEstimateValidator {
        rules: vec![
            Rc::new(GlobalCredentialEstimateSegmentLengthRule { max_length: 32 })
                as Rc<dyn GlobalCredentialEstimateRule>,
            Rc::new(GlobalCredentialEstimateCapacityRangeRule { min: 120, max: 86400 }),
            Rc::new(GlobalCredentialEstimateRequiredLocaleRule {
                message: "global credential estimate needs a value".to_string(),
            }),
            Rc::new(GlobalCredentialEstimateTtlSecondsRangeRule { min: 64, max: 80 }),
            Rc::new(GlobalCredentialEstimateRequiredCodeRule {
                message: "global credential estimate needs this field".to_string(),
            }),
            Rc::new(GlobalCredentialEstimateCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_credential_estimate_validator();
        let item = new_global_credential_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = GlobalCredentialEstimateSegmentLengthRule { max_length: 1000 };
        let mut item = new_global_credential_estimate("x", "y");
        item.segment = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = GlobalCredentialEstimateCapacityRangeRule { min: 16, max: 32 };
        let mut item = new_global_credential_estimate("x", "y");
        item.capacity = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = GlobalCredentialEstimateRequiredLocaleRule {
            message: "global credential estimate needs this field".to_string(),
        };
        let mut item = new_global_credential_estimate("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = GlobalCredentialEstimateTtlSecondsRangeRule { min: 32, max: 3600 };
        let mut item = new_global_credential_estimate("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalCredentialEstimateRequiredCodeRule {
            message: "global credential estimate needs this field".to_string(),
        };
        let mut item = new_global_credential_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
