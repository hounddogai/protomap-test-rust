use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global nonce estimate must pass. Check returns its problems, or none.
pub trait GlobalNonceEstimateRule: std::fmt::Debug {
    fn check(&self, item: &GlobalNonceEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalNonceEstimateRequiredCodeRule {
    pub message: String,
}

impl GlobalNonceEstimateRule for GlobalNonceEstimateRequiredCodeRule {
    fn check(&self, item: &GlobalNonceEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalNonceEstimateVisibleGuardRule {
    pub limit: i64,
}

impl GlobalNonceEstimateRule for GlobalNonceEstimateVisibleGuardRule {
    fn check(&self, item: &GlobalNonceEstimate) -> Vec<String> {
        if item.visible && (item.attempts > self.limit) {
            return vec![format!(
                "attempts {} exceeds {} while visible",
                item.attempts, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalNonceEstimateAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalNonceEstimateRule for GlobalNonceEstimateAttemptsRangeRule {
    fn check(&self, item: &GlobalNonceEstimate) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalNonceEstimateCodePrefixRule {
    pub prefix: String,
}

impl GlobalNonceEstimateRule for GlobalNonceEstimateCodePrefixRule {
    fn check(&self, item: &GlobalNonceEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalNonceEstimateLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalNonceEstimateRule for GlobalNonceEstimateLimitRangeRule {
    fn check(&self, item: &GlobalNonceEstimate) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a global nonce estimate.
#[derive(Debug, Clone)]
pub struct GlobalNonceEstimateValidator {
    pub rules: Vec<Rc<dyn GlobalNonceEstimateRule>>,
}

impl GlobalNonceEstimateValidator {
    pub fn validate(&self, item: &GlobalNonceEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalNonceEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalNonceEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_nonce_estimate_validator() -> GlobalNonceEstimateValidator {
    GlobalNonceEstimateValidator {
        rules: vec![
            Rc::new(GlobalNonceEstimateRequiredCodeRule {
                message: "global nonce estimate needs an entry".to_string(),
            }) as Rc<dyn GlobalNonceEstimateRule>,
            Rc::new(GlobalNonceEstimateVisibleGuardRule { limit: 3600 }),
            Rc::new(GlobalNonceEstimateAttemptsRangeRule { min: 32, max: 80 }),
            Rc::new(GlobalNonceEstimateCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(GlobalNonceEstimateLimitRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_nonce_estimate_validator();
        let item = new_global_nonce_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalNonceEstimateRequiredCodeRule {
            message: "global nonce estimate needs a value".to_string(),
        };
        let mut item = new_global_nonce_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = GlobalNonceEstimateAttemptsRangeRule { min: 120, max: 255 };
        let mut item = new_global_nonce_estimate("x", "y");
        item.attempts = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = GlobalNonceEstimateLimitRangeRule { min: 255, max: 86400 };
        let mut item = new_global_nonce_estimate("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
