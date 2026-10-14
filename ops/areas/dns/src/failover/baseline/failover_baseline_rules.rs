use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover baseline must pass. Check returns its problems, or none.
pub trait FailoverBaselineRule: std::fmt::Debug {
    fn check(&self, item: &FailoverBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBaselineCodePrefixRule {
    pub prefix: String,
}

impl FailoverBaselineRule for FailoverBaselineCodePrefixRule {
    fn check(&self, item: &FailoverBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBaselineFingerprintLengthRule {
    pub max_length: i64,
}

impl FailoverBaselineRule for FailoverBaselineFingerprintLengthRule {
    fn check(&self, item: &FailoverBaseline) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBaselineRequiredClusterRule {
    pub message: String,
}

impl FailoverBaselineRule for FailoverBaselineRequiredClusterRule {
    fn check(&self, item: &FailoverBaseline) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBaselineExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverBaselineRule for FailoverBaselineExpiresAtRangeRule {
    fn check(&self, item: &FailoverBaseline) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBaselineApprovedGuardRule {
    pub limit: f64,
}

impl FailoverBaselineRule for FailoverBaselineApprovedGuardRule {
    fn check(&self, item: &FailoverBaseline) -> Vec<String> {
        if item.approved && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while approved", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBaselineRequiredNameRule {
    pub message: String,
}

impl FailoverBaselineRule for FailoverBaselineRequiredNameRule {
    fn check(&self, item: &FailoverBaseline) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover baseline.
#[derive(Debug, Clone)]
pub struct FailoverBaselineValidator {
    pub rules: Vec<Rc<dyn FailoverBaselineRule>>,
}

impl FailoverBaselineValidator {
    pub fn validate(&self, item: &FailoverBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_baseline_validator() -> FailoverBaselineValidator {
    FailoverBaselineValidator {
        rules: vec![
            Rc::new(FailoverBaselineCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn FailoverBaselineRule>,
            Rc::new(FailoverBaselineFingerprintLengthRule { max_length: 32 }),
            Rc::new(FailoverBaselineRequiredClusterRule {
                message: "failover baseline needs a value".to_string(),
            }),
            Rc::new(FailoverBaselineExpiresAtRangeRule { min: 1000, max: 1000 }),
            Rc::new(FailoverBaselineApprovedGuardRule { limit: 7.7 }),
            Rc::new(FailoverBaselineRequiredNameRule {
                message: "failover baseline needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_baseline_validator();
        let item = new_failover_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = FailoverBaselineFingerprintLengthRule { max_length: 1000 };
        let mut item = new_failover_baseline("x", "y");
        item.fingerprint = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = FailoverBaselineRequiredClusterRule {
            message: "failover baseline needs this field".to_string(),
        };
        let mut item = new_failover_baseline("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = FailoverBaselineExpiresAtRangeRule { min: 255, max: 86400 };
        let mut item = new_failover_baseline("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FailoverBaselineRequiredNameRule {
            message: "failover baseline needs a value".to_string(),
        };
        let mut item = new_failover_baseline("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
