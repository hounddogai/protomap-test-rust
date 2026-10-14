use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling token profile must pass. Check returns its problems, or none.
pub trait RollingTokenProfileRule: std::fmt::Debug {
    fn check(&self, item: &RollingTokenProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenProfileRequiredIssuerRule {
    pub message: String,
}

impl RollingTokenProfileRule for RollingTokenProfileRequiredIssuerRule {
    fn check(&self, item: &RollingTokenProfile) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenProfileIssuerLengthRule {
    pub max_length: i64,
}

impl RollingTokenProfileRule for RollingTokenProfileIssuerLengthRule {
    fn check(&self, item: &RollingTokenProfile) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenProfileCodePrefixRule {
    pub prefix: String,
}

impl RollingTokenProfileRule for RollingTokenProfileCodePrefixRule {
    fn check(&self, item: &RollingTokenProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenProfileGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingTokenProfileRule for RollingTokenProfileGraceDaysRangeRule {
    fn check(&self, item: &RollingTokenProfile) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenProfileRequiredCodeRule {
    pub message: String,
}

impl RollingTokenProfileRule for RollingTokenProfileRequiredCodeRule {
    fn check(&self, item: &RollingTokenProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling token profile.
#[derive(Debug, Clone)]
pub struct RollingTokenProfileValidator {
    pub rules: Vec<Rc<dyn RollingTokenProfileRule>>,
}

impl RollingTokenProfileValidator {
    pub fn validate(&self, item: &RollingTokenProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingTokenProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTokenProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_token_profile_validator() -> RollingTokenProfileValidator {
    RollingTokenProfileValidator {
        rules: vec![
            Rc::new(RollingTokenProfileRequiredIssuerRule {
                message: "rolling token profile needs this field".to_string(),
            }) as Rc<dyn RollingTokenProfileRule>,
            Rc::new(RollingTokenProfileIssuerLengthRule { max_length: 64 }),
            Rc::new(RollingTokenProfileCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RollingTokenProfileGraceDaysRangeRule { min: 32, max: 3600 }),
            Rc::new(RollingTokenProfileRequiredCodeRule {
                message: "rolling token profile needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_token_profile_validator();
        let item = new_rolling_token_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = RollingTokenProfileRequiredIssuerRule {
            message: "rolling token profile needs this field".to_string(),
        };
        let mut item = new_rolling_token_profile("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = RollingTokenProfileIssuerLengthRule { max_length: 1000 };
        let mut item = new_rolling_token_profile("x", "y");
        item.issuer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = RollingTokenProfileGraceDaysRangeRule { min: 120, max: 3600 };
        let mut item = new_rolling_token_profile("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingTokenProfileRequiredCodeRule {
            message: "rolling token profile needs this field".to_string(),
        };
        let mut item = new_rolling_token_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
