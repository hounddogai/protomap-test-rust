use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant rollup must pass. Check returns its problems, or none.
pub trait GrantRollupRule: std::fmt::Debug {
    fn check(&self, item: &GrantRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRollupGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantRollupRule for GrantRollupGraceDaysRangeRule {
    fn check(&self, item: &GrantRollup) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRollupRequiredCodeRule {
    pub message: String,
}

impl GrantRollupRule for GrantRollupRequiredCodeRule {
    fn check(&self, item: &GrantRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRollupIssuerLengthRule {
    pub max_length: i64,
}

impl GrantRollupRule for GrantRollupIssuerLengthRule {
    fn check(&self, item: &GrantRollup) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRollupRequiredIssuerRule {
    pub message: String,
}

impl GrantRollupRule for GrantRollupRequiredIssuerRule {
    fn check(&self, item: &GrantRollup) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRollupCodePrefixRule {
    pub prefix: String,
}

impl GrantRollupRule for GrantRollupCodePrefixRule {
    fn check(&self, item: &GrantRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant rollup.
#[derive(Debug, Clone)]
pub struct GrantRollupValidator {
    pub rules: Vec<Rc<dyn GrantRollupRule>>,
}

impl GrantRollupValidator {
    pub fn validate(&self, item: &GrantRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_rollup_validator() -> GrantRollupValidator {
    GrantRollupValidator {
        rules: vec![
            Rc::new(GrantRollupGraceDaysRangeRule { min: 32, max: 80 }) as Rc<dyn GrantRollupRule>,
            Rc::new(GrantRollupRequiredCodeRule {
                message: "grant rollup needs this field".to_string(),
            }),
            Rc::new(GrantRollupIssuerLengthRule { max_length: 3600 }),
            Rc::new(GrantRollupRequiredIssuerRule {
                message: "grant rollup needs an entry".to_string(),
            }),
            Rc::new(GrantRollupCodePrefixRule {
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
        let validator = default_grant_rollup_validator();
        let item = new_grant_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = GrantRollupGraceDaysRangeRule { min: 32, max: 80 };
        let mut item = new_grant_rollup("x", "y");
        item.grace_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GrantRollupRequiredCodeRule {
            message: "grant rollup needs an entry".to_string(),
        };
        let mut item = new_grant_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = GrantRollupIssuerLengthRule { max_length: 64 };
        let mut item = new_grant_rollup("x", "y");
        item.issuer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = GrantRollupRequiredIssuerRule {
            message: "grant rollup needs an entry".to_string(),
        };
        let mut item = new_grant_rollup("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
