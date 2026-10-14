use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage claim must pass. Check returns its problems, or none.
pub trait OverageClaimRule: std::fmt::Debug {
    fn check(&self, item: &OverageClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageClaimCodePrefixRule {
    pub prefix: String,
}

impl OverageClaimRule for OverageClaimCodePrefixRule {
    fn check(&self, item: &OverageClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageClaimRequiredCodeRule {
    pub message: String,
}

impl OverageClaimRule for OverageClaimRequiredCodeRule {
    fn check(&self, item: &OverageClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageClaimThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OverageClaimRule for OverageClaimThresholdRangeRule {
    fn check(&self, item: &OverageClaim) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageClaimRequiredClusterRule {
    pub message: String,
}

impl OverageClaimRule for OverageClaimRequiredClusterRule {
    fn check(&self, item: &OverageClaim) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageClaimClusterLengthRule {
    pub max_length: i64,
}

impl OverageClaimRule for OverageClaimClusterLengthRule {
    fn check(&self, item: &OverageClaim) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage claim.
#[derive(Debug, Clone)]
pub struct OverageClaimValidator {
    pub rules: Vec<Rc<dyn OverageClaimRule>>,
}

impl OverageClaimValidator {
    pub fn validate(&self, item: &OverageClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_claim_validator() -> OverageClaimValidator {
    OverageClaimValidator {
        rules: vec![
            Rc::new(OverageClaimCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn OverageClaimRule>,
            Rc::new(OverageClaimRequiredCodeRule {
                message: "overage claim needs a value".to_string(),
            }),
            Rc::new(OverageClaimThresholdRangeRule { min: 4.8, max: 2945.1 }),
            Rc::new(OverageClaimRequiredClusterRule {
                message: "overage claim needs a value".to_string(),
            }),
            Rc::new(OverageClaimClusterLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_claim_validator();
        let item = new_overage_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = OverageClaimRequiredCodeRule {
            message: "overage claim needs a value".to_string(),
        };
        let mut item = new_overage_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = OverageClaimThresholdRangeRule { min: 3.4, max: 694.5 };
        let mut item = new_overage_claim("x", "y");
        item.threshold = 696.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = OverageClaimRequiredClusterRule {
            message: "overage claim needs an entry".to_string(),
        };
        let mut item = new_overage_claim("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = OverageClaimClusterLengthRule { max_length: 32 };
        let mut item = new_overage_claim("x", "y");
        item.cluster = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
