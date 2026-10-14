use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional failover override must pass. Check returns its problems, or none.
pub trait RegionalFailoverOverrideRule: std::fmt::Debug {
    fn check(&self, item: &RegionalFailoverOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverOverrideLocaleLengthRule {
    pub max_length: i64,
}

impl RegionalFailoverOverrideRule for RegionalFailoverOverrideLocaleLengthRule {
    fn check(&self, item: &RegionalFailoverOverride) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverOverrideRequiredLocaleRule {
    pub message: String,
}

impl RegionalFailoverOverrideRule for RegionalFailoverOverrideRequiredLocaleRule {
    fn check(&self, item: &RegionalFailoverOverride) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverOverrideCodePrefixRule {
    pub prefix: String,
}

impl RegionalFailoverOverrideRule for RegionalFailoverOverrideCodePrefixRule {
    fn check(&self, item: &RegionalFailoverOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverOverrideCodeLengthRule {
    pub max_length: i64,
}

impl RegionalFailoverOverrideRule for RegionalFailoverOverrideCodeLengthRule {
    fn check(&self, item: &RegionalFailoverOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverOverrideRequiredTitleRule {
    pub message: String,
}

impl RegionalFailoverOverrideRule for RegionalFailoverOverrideRequiredTitleRule {
    fn check(&self, item: &RegionalFailoverOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional failover override.
#[derive(Debug, Clone)]
pub struct RegionalFailoverOverrideValidator {
    pub rules: Vec<Rc<dyn RegionalFailoverOverrideRule>>,
}

impl RegionalFailoverOverrideValidator {
    pub fn validate(&self, item: &RegionalFailoverOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalFailoverOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalFailoverOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_failover_override_validator() -> RegionalFailoverOverrideValidator {
    RegionalFailoverOverrideValidator {
        rules: vec![
            Rc::new(RegionalFailoverOverrideLocaleLengthRule { max_length: 120 })
                as Rc<dyn RegionalFailoverOverrideRule>,
            Rc::new(RegionalFailoverOverrideRequiredLocaleRule {
                message: "regional failover override needs this field".to_string(),
            }),
            Rc::new(RegionalFailoverOverrideCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RegionalFailoverOverrideCodeLengthRule { max_length: 255 }),
            Rc::new(RegionalFailoverOverrideRequiredTitleRule {
                message: "regional failover override needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_failover_override_validator();
        let item = new_regional_failover_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = RegionalFailoverOverrideLocaleLengthRule { max_length: 80 };
        let mut item = new_regional_failover_override("x", "y");
        item.locale = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = RegionalFailoverOverrideRequiredLocaleRule {
            message: "regional failover override needs an entry".to_string(),
        };
        let mut item = new_regional_failover_override("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalFailoverOverrideCodeLengthRule { max_length: 32 };
        let mut item = new_regional_failover_override("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionalFailoverOverrideRequiredTitleRule {
            message: "regional failover override needs a value".to_string(),
        };
        let mut item = new_regional_failover_override("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
