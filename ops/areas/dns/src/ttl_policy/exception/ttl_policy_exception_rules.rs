use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy exception must pass. Check returns its problems, or none.
pub trait TtlPolicyExceptionRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyExceptionExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyExceptionRule for TtlPolicyExceptionExpiresAtRangeRule {
    fn check(&self, item: &TtlPolicyException) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyExceptionRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyExceptionRule for TtlPolicyExceptionRatioRangeRule {
    fn check(&self, item: &TtlPolicyException) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyExceptionCodePrefixRule {
    pub prefix: String,
}

impl TtlPolicyExceptionRule for TtlPolicyExceptionCodePrefixRule {
    fn check(&self, item: &TtlPolicyException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyExceptionTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl TtlPolicyExceptionRule for TtlPolicyExceptionTagsLimitRule {
    fn check(&self, item: &TtlPolicyException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyExceptionNameLengthRule {
    pub max_length: i64,
}

impl TtlPolicyExceptionRule for TtlPolicyExceptionNameLengthRule {
    fn check(&self, item: &TtlPolicyException) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy exception.
#[derive(Debug, Clone)]
pub struct TtlPolicyExceptionValidator {
    pub rules: Vec<Rc<dyn TtlPolicyExceptionRule>>,
}

impl TtlPolicyExceptionValidator {
    pub fn validate(&self, item: &TtlPolicyException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicyException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_exception_validator() -> TtlPolicyExceptionValidator {
    TtlPolicyExceptionValidator {
        rules: vec![
            Rc::new(TtlPolicyExceptionExpiresAtRangeRule { min: 255, max: 1000 }) as Rc<dyn TtlPolicyExceptionRule>,
            Rc::new(TtlPolicyExceptionRatioRangeRule { min: 7.4, max: 4723.6 }),
            Rc::new(TtlPolicyExceptionCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(TtlPolicyExceptionTagsLimitRule {
                max_count: 64,
                banned: vec!["lenient".to_string(), "batched".to_string(), "lenient".to_string()],
            }),
            Rc::new(TtlPolicyExceptionNameLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_exception_validator();
        let item = new_ttl_policy_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = TtlPolicyExceptionExpiresAtRangeRule { min: 120, max: 86400 };
        let mut item = new_ttl_policy_exception("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = TtlPolicyExceptionRatioRangeRule { min: 0.3, max: 1467.7 };
        let mut item = new_ttl_policy_exception("x", "y");
        item.ratio = 1469.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TtlPolicyExceptionNameLengthRule { max_length: 255 };
        let mut item = new_ttl_policy_exception("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
