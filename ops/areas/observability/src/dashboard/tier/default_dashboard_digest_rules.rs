use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default dashboard digest must pass. Check returns its problems, or none.
pub trait DefaultDashboardDigestRule: std::fmt::Debug {
    fn check(&self, item: &DefaultDashboardDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardDigestRequiredCurrencyRule {
    pub message: String,
}

impl DefaultDashboardDigestRule for DefaultDashboardDigestRequiredCurrencyRule {
    fn check(&self, item: &DefaultDashboardDigest) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardDigestRequiredCodeRule {
    pub message: String,
}

impl DefaultDashboardDigestRule for DefaultDashboardDigestRequiredCodeRule {
    fn check(&self, item: &DefaultDashboardDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardDigestCodePrefixRule {
    pub prefix: String,
}

impl DefaultDashboardDigestRule for DefaultDashboardDigestCodePrefixRule {
    fn check(&self, item: &DefaultDashboardDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardDigestVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultDashboardDigestRule for DefaultDashboardDigestVersionRangeRule {
    fn check(&self, item: &DefaultDashboardDigest) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardDigestCodeLengthRule {
    pub max_length: i64,
}

impl DefaultDashboardDigestRule for DefaultDashboardDigestCodeLengthRule {
    fn check(&self, item: &DefaultDashboardDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardDigestRequiredTitleRule {
    pub message: String,
}

impl DefaultDashboardDigestRule for DefaultDashboardDigestRequiredTitleRule {
    fn check(&self, item: &DefaultDashboardDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default dashboard digest.
#[derive(Debug, Clone)]
pub struct DefaultDashboardDigestValidator {
    pub rules: Vec<Rc<dyn DefaultDashboardDigestRule>>,
}

impl DefaultDashboardDigestValidator {
    pub fn validate(&self, item: &DefaultDashboardDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultDashboardDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultDashboardDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_dashboard_digest_validator() -> DefaultDashboardDigestValidator {
    DefaultDashboardDigestValidator {
        rules: vec![
            Rc::new(DefaultDashboardDigestRequiredCurrencyRule {
                message: "default dashboard digest needs an entry".to_string(),
            }) as Rc<dyn DefaultDashboardDigestRule>,
            Rc::new(DefaultDashboardDigestRequiredCodeRule {
                message: "default dashboard digest needs an entry".to_string(),
            }),
            Rc::new(DefaultDashboardDigestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(DefaultDashboardDigestVersionRangeRule { min: 80, max: 1000 }),
            Rc::new(DefaultDashboardDigestCodeLengthRule { max_length: 80 }),
            Rc::new(DefaultDashboardDigestRequiredTitleRule {
                message: "default dashboard digest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_dashboard_digest_validator();
        let item = new_default_dashboard_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = DefaultDashboardDigestRequiredCurrencyRule {
            message: "default dashboard digest needs this field".to_string(),
        };
        let mut item = new_default_dashboard_digest("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultDashboardDigestRequiredCodeRule {
            message: "default dashboard digest needs this field".to_string(),
        };
        let mut item = new_default_dashboard_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = DefaultDashboardDigestVersionRangeRule { min: 16, max: 80 };
        let mut item = new_default_dashboard_digest("x", "y");
        item.version = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultDashboardDigestCodeLengthRule { max_length: 3600 };
        let mut item = new_default_dashboard_digest("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DefaultDashboardDigestRequiredTitleRule {
            message: "default dashboard digest needs an entry".to_string(),
        };
        let mut item = new_default_dashboard_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
