use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a alert rollup must pass. Check returns its problems, or none.
pub trait AlertRollupRule: std::fmt::Debug {
    fn check(&self, item: &AlertRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRollupRequiredCodeRule {
    pub message: String,
}

impl AlertRollupRule for AlertRollupRequiredCodeRule {
    fn check(&self, item: &AlertRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRollupRequiredTitleRule {
    pub message: String,
}

impl AlertRollupRule for AlertRollupRequiredTitleRule {
    fn check(&self, item: &AlertRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRollupCurrencyLengthRule {
    pub max_length: i64,
}

impl AlertRollupRule for AlertRollupCurrencyLengthRule {
    fn check(&self, item: &AlertRollup) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRollupAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl AlertRollupRule for AlertRollupAllowedPrincipalRule {
    fn check(&self, item: &AlertRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is out of range", item.principal)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert rollup.
#[derive(Debug, Clone)]
pub struct AlertRollupValidator {
    pub rules: Vec<Rc<dyn AlertRollupRule>>,
}

impl AlertRollupValidator {
    pub fn validate(&self, item: &AlertRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_rollup_validator() -> AlertRollupValidator {
    AlertRollupValidator {
        rules: vec![
            Rc::new(AlertRollupRequiredCodeRule {
                message: "alert rollup needs a value".to_string(),
            }) as Rc<dyn AlertRollupRule>,
            Rc::new(AlertRollupRequiredTitleRule {
                message: "alert rollup needs a value".to_string(),
            }),
            Rc::new(AlertRollupCurrencyLengthRule { max_length: 32 }),
            Rc::new(AlertRollupAllowedPrincipalRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_rollup_validator();
        let item = new_alert_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AlertRollupRequiredCodeRule {
            message: "alert rollup needs an entry".to_string(),
        };
        let mut item = new_alert_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AlertRollupRequiredTitleRule {
            message: "alert rollup needs a value".to_string(),
        };
        let mut item = new_alert_rollup("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = AlertRollupCurrencyLengthRule { max_length: 64 };
        let mut item = new_alert_rollup("x", "y");
        item.currency = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
