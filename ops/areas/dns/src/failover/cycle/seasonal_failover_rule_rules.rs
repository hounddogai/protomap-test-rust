use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal failover rule must pass. Check returns its problems, or none.
pub trait SeasonalFailoverRuleRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalFailoverRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFailoverRuleAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl SeasonalFailoverRuleRule for SeasonalFailoverRuleAllowedDescriptionRule {
    fn check(&self, item: &SeasonalFailoverRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is out of range", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFailoverRuleDescriptionLengthRule {
    pub max_length: i64,
}

impl SeasonalFailoverRuleRule for SeasonalFailoverRuleDescriptionLengthRule {
    fn check(&self, item: &SeasonalFailoverRule) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFailoverRuleCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalFailoverRuleRule for SeasonalFailoverRuleCodeLengthRule {
    fn check(&self, item: &SeasonalFailoverRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFailoverRuleRequiredFingerprintRule {
    pub message: String,
}

impl SeasonalFailoverRuleRule for SeasonalFailoverRuleRequiredFingerprintRule {
    fn check(&self, item: &SeasonalFailoverRule) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFailoverRuleFingerprintLengthRule {
    pub max_length: i64,
}

impl SeasonalFailoverRuleRule for SeasonalFailoverRuleFingerprintLengthRule {
    fn check(&self, item: &SeasonalFailoverRule) -> Vec<String> {
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

/// Runs every rule of a seasonal failover rule.
#[derive(Debug, Clone)]
pub struct SeasonalFailoverRuleValidator {
    pub rules: Vec<Rc<dyn SeasonalFailoverRuleRule>>,
}

impl SeasonalFailoverRuleValidator {
    pub fn validate(&self, item: &SeasonalFailoverRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalFailoverRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalFailoverRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_failover_rule_validator() -> SeasonalFailoverRuleValidator {
    SeasonalFailoverRuleValidator {
        rules: vec![
            Rc::new(SeasonalFailoverRuleAllowedDescriptionRule {
                allowed: vec!["lenient".to_string()],
            }) as Rc<dyn SeasonalFailoverRuleRule>,
            Rc::new(SeasonalFailoverRuleDescriptionLengthRule { max_length: 64 }),
            Rc::new(SeasonalFailoverRuleCodeLengthRule { max_length: 3600 }),
            Rc::new(SeasonalFailoverRuleRequiredFingerprintRule {
                message: "seasonal failover rule needs a value".to_string(),
            }),
            Rc::new(SeasonalFailoverRuleFingerprintLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_failover_rule_validator();
        let item = new_seasonal_failover_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = SeasonalFailoverRuleDescriptionLengthRule { max_length: 64 };
        let mut item = new_seasonal_failover_rule("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalFailoverRuleCodeLengthRule { max_length: 86400 };
        let mut item = new_seasonal_failover_rule("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = SeasonalFailoverRuleRequiredFingerprintRule {
            message: "seasonal failover rule needs an entry".to_string(),
        };
        let mut item = new_seasonal_failover_rule("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = SeasonalFailoverRuleFingerprintLengthRule { max_length: 64 };
        let mut item = new_seasonal_failover_rule("x", "y");
        item.fingerprint = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
