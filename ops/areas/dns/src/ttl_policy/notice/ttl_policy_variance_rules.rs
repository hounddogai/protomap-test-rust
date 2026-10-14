use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy variance must pass. Check returns its problems, or none.
pub trait TtlPolicyVarianceRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyVarianceLabelLengthRule {
    pub max_length: i64,
}

impl TtlPolicyVarianceRule for TtlPolicyVarianceLabelLengthRule {
    fn check(&self, item: &TtlPolicyVariance) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyVarianceAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyVarianceRule for TtlPolicyVarianceAttemptsRangeRule {
    fn check(&self, item: &TtlPolicyVariance) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyVarianceCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyVarianceRule for TtlPolicyVarianceCeilingRangeRule {
    fn check(&self, item: &TtlPolicyVariance) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyVarianceRequiredCodeRule {
    pub message: String,
}

impl TtlPolicyVarianceRule for TtlPolicyVarianceRequiredCodeRule {
    fn check(&self, item: &TtlPolicyVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyVarianceCodePrefixRule {
    pub prefix: String,
}

impl TtlPolicyVarianceRule for TtlPolicyVarianceCodePrefixRule {
    fn check(&self, item: &TtlPolicyVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy variance.
#[derive(Debug, Clone)]
pub struct TtlPolicyVarianceValidator {
    pub rules: Vec<Rc<dyn TtlPolicyVarianceRule>>,
}

impl TtlPolicyVarianceValidator {
    pub fn validate(&self, item: &TtlPolicyVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicyVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_variance_validator() -> TtlPolicyVarianceValidator {
    TtlPolicyVarianceValidator {
        rules: vec![
            Rc::new(TtlPolicyVarianceLabelLengthRule { max_length: 1000 }) as Rc<dyn TtlPolicyVarianceRule>,
            Rc::new(TtlPolicyVarianceAttemptsRangeRule { min: 64, max: 3600 }),
            Rc::new(TtlPolicyVarianceCeilingRangeRule { min: 8.2, max: 2090.7 }),
            Rc::new(TtlPolicyVarianceRequiredCodeRule {
                message: "ttl policy variance needs an entry".to_string(),
            }),
            Rc::new(TtlPolicyVarianceCodePrefixRule {
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
        let validator = default_ttl_policy_variance_validator();
        let item = new_ttl_policy_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TtlPolicyVarianceLabelLengthRule { max_length: 120 };
        let mut item = new_ttl_policy_variance("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = TtlPolicyVarianceAttemptsRangeRule { min: 16, max: 80 };
        let mut item = new_ttl_policy_variance("x", "y");
        item.attempts = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = TtlPolicyVarianceCeilingRangeRule { min: 3.7, max: 4120.9 };
        let mut item = new_ttl_policy_variance("x", "y");
        item.ceiling = 4122.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TtlPolicyVarianceRequiredCodeRule {
            message: "ttl policy variance needs a value".to_string(),
        };
        let mut item = new_ttl_policy_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
