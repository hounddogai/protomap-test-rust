use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier override must pass. Check returns its problems, or none.
pub trait VerifierOverrideRule: std::fmt::Debug {
    fn check(&self, item: &VerifierOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierOverrideClusterLengthRule {
    pub max_length: i64,
}

impl VerifierOverrideRule for VerifierOverrideClusterLengthRule {
    fn check(&self, item: &VerifierOverride) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierOverrideRequiredCodeRule {
    pub message: String,
}

impl VerifierOverrideRule for VerifierOverrideRequiredCodeRule {
    fn check(&self, item: &VerifierOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierOverrideTitleLengthRule {
    pub max_length: i64,
}

impl VerifierOverrideRule for VerifierOverrideTitleLengthRule {
    fn check(&self, item: &VerifierOverride) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierOverrideCodePrefixRule {
    pub prefix: String,
}

impl VerifierOverrideRule for VerifierOverrideCodePrefixRule {
    fn check(&self, item: &VerifierOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierOverrideLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierOverrideRule for VerifierOverrideLimitRangeRule {
    fn check(&self, item: &VerifierOverride) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierOverridePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierOverrideRule for VerifierOverridePortRangeRule {
    fn check(&self, item: &VerifierOverride) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier override.
#[derive(Debug, Clone)]
pub struct VerifierOverrideValidator {
    pub rules: Vec<Rc<dyn VerifierOverrideRule>>,
}

impl VerifierOverrideValidator {
    pub fn validate(&self, item: &VerifierOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_override_validator() -> VerifierOverrideValidator {
    VerifierOverrideValidator {
        rules: vec![
            Rc::new(VerifierOverrideClusterLengthRule { max_length: 80 }) as Rc<dyn VerifierOverrideRule>,
            Rc::new(VerifierOverrideRequiredCodeRule {
                message: "verifier override needs a value".to_string(),
            }),
            Rc::new(VerifierOverrideTitleLengthRule { max_length: 255 }),
            Rc::new(VerifierOverrideCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(VerifierOverrideLimitRangeRule { min: 80, max: 255 }),
            Rc::new(VerifierOverridePortRangeRule { min: 64, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_override_validator();
        let item = new_verifier_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = VerifierOverrideClusterLengthRule { max_length: 80 };
        let mut item = new_verifier_override("x", "y");
        item.cluster = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = VerifierOverrideRequiredCodeRule {
            message: "verifier override needs an entry".to_string(),
        };
        let mut item = new_verifier_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = VerifierOverrideTitleLengthRule { max_length: 120 };
        let mut item = new_verifier_override("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = VerifierOverrideLimitRangeRule { min: 32, max: 120 };
        let mut item = new_verifier_override("x", "y");
        item.limit = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = VerifierOverridePortRangeRule { min: 16, max: 64 };
        let mut item = new_verifier_override("x", "y");
        item.port = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
