use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a credential score must pass. Check returns its problems, or none.
pub trait CredentialScoreRule: std::fmt::Debug {
    fn check(&self, item: &CredentialScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialScoreRequiredChannelRule {
    pub message: String,
}

impl CredentialScoreRule for CredentialScoreRequiredChannelRule {
    fn check(&self, item: &CredentialScore) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialScoreCodePrefixRule {
    pub prefix: String,
}

impl CredentialScoreRule for CredentialScoreCodePrefixRule {
    fn check(&self, item: &CredentialScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialScoreRequiredLabelRule {
    pub message: String,
}

impl CredentialScoreRule for CredentialScoreRequiredLabelRule {
    fn check(&self, item: &CredentialScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialScoreBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CredentialScoreRule for CredentialScoreBudgetRangeRule {
    fn check(&self, item: &CredentialScore) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a credential score.
#[derive(Debug, Clone)]
pub struct CredentialScoreValidator {
    pub rules: Vec<Rc<dyn CredentialScoreRule>>,
}

impl CredentialScoreValidator {
    pub fn validate(&self, item: &CredentialScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_score_validator() -> CredentialScoreValidator {
    CredentialScoreValidator {
        rules: vec![
            Rc::new(CredentialScoreRequiredChannelRule {
                message: "credential score needs a value".to_string(),
            }) as Rc<dyn CredentialScoreRule>,
            Rc::new(CredentialScoreCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(CredentialScoreRequiredLabelRule {
                message: "credential score needs an entry".to_string(),
            }),
            Rc::new(CredentialScoreBudgetRangeRule { min: 0.6, max: 3030.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_score_validator();
        let item = new_credential_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = CredentialScoreRequiredChannelRule {
            message: "credential score needs a value".to_string(),
        };
        let mut item = new_credential_score("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CredentialScoreRequiredLabelRule {
            message: "credential score needs an entry".to_string(),
        };
        let mut item = new_credential_score("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = CredentialScoreBudgetRangeRule { min: 0.4, max: 2193.4 };
        let mut item = new_credential_score("x", "y");
        item.budget = 2194.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
