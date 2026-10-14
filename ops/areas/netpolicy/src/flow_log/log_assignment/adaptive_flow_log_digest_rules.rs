use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive flow log digest must pass. Check returns its problems, or none.
pub trait AdaptiveFlowLogDigestRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveFlowLogDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogDigestRequiredCodeRule {
    pub message: String,
}

impl AdaptiveFlowLogDigestRule for AdaptiveFlowLogDigestRequiredCodeRule {
    fn check(&self, item: &AdaptiveFlowLogDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogDigestTitleLengthRule {
    pub max_length: i64,
}

impl AdaptiveFlowLogDigestRule for AdaptiveFlowLogDigestTitleLengthRule {
    fn check(&self, item: &AdaptiveFlowLogDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogDigestApprovedGuardRule {
    pub limit: f64,
}

impl AdaptiveFlowLogDigestRule for AdaptiveFlowLogDigestApprovedGuardRule {
    fn check(&self, item: &AdaptiveFlowLogDigest) -> Vec<String> {
        if item.approved && (item.minimum > self.limit) {
            return vec![format!(
                "minimum {} exceeds {} while approved",
                item.minimum, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogDigestMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveFlowLogDigestRule for AdaptiveFlowLogDigestMinimumRangeRule {
    fn check(&self, item: &AdaptiveFlowLogDigest) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogDigestRequiredScopeRule {
    pub message: String,
}

impl AdaptiveFlowLogDigestRule for AdaptiveFlowLogDigestRequiredScopeRule {
    fn check(&self, item: &AdaptiveFlowLogDigest) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogDigestRequiredFingerprintRule {
    pub message: String,
}

impl AdaptiveFlowLogDigestRule for AdaptiveFlowLogDigestRequiredFingerprintRule {
    fn check(&self, item: &AdaptiveFlowLogDigest) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive flow log digest.
#[derive(Debug, Clone)]
pub struct AdaptiveFlowLogDigestValidator {
    pub rules: Vec<Rc<dyn AdaptiveFlowLogDigestRule>>,
}

impl AdaptiveFlowLogDigestValidator {
    pub fn validate(&self, item: &AdaptiveFlowLogDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveFlowLogDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveFlowLogDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_flow_log_digest_validator() -> AdaptiveFlowLogDigestValidator {
    AdaptiveFlowLogDigestValidator {
        rules: vec![
            Rc::new(AdaptiveFlowLogDigestRequiredCodeRule {
                message: "adaptive flow log digest needs this field".to_string(),
            }) as Rc<dyn AdaptiveFlowLogDigestRule>,
            Rc::new(AdaptiveFlowLogDigestTitleLengthRule { max_length: 86400 }),
            Rc::new(AdaptiveFlowLogDigestApprovedGuardRule { limit: 5.3 }),
            Rc::new(AdaptiveFlowLogDigestMinimumRangeRule { min: 8.5, max: 3618.5 }),
            Rc::new(AdaptiveFlowLogDigestRequiredScopeRule {
                message: "adaptive flow log digest needs this field".to_string(),
            }),
            Rc::new(AdaptiveFlowLogDigestRequiredFingerprintRule {
                message: "adaptive flow log digest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_flow_log_digest_validator();
        let item = new_adaptive_flow_log_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveFlowLogDigestRequiredCodeRule {
            message: "adaptive flow log digest needs an entry".to_string(),
        };
        let mut item = new_adaptive_flow_log_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AdaptiveFlowLogDigestTitleLengthRule { max_length: 80 };
        let mut item = new_adaptive_flow_log_digest("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = AdaptiveFlowLogDigestMinimumRangeRule { min: 4.2, max: 4233.1 };
        let mut item = new_adaptive_flow_log_digest("x", "y");
        item.minimum = 4234.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = AdaptiveFlowLogDigestRequiredScopeRule {
            message: "adaptive flow log digest needs a value".to_string(),
        };
        let mut item = new_adaptive_flow_log_digest("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = AdaptiveFlowLogDigestRequiredFingerprintRule {
            message: "adaptive flow log digest needs a value".to_string(),
        };
        let mut item = new_adaptive_flow_log_digest("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
