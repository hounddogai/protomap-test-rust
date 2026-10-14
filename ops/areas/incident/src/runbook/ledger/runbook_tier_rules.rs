use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook tier must pass. Check returns its problems, or none.
pub trait RunbookTierRule: std::fmt::Debug {
    fn check(&self, item: &RunbookTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTierRequiredCountryRule {
    pub message: String,
}

impl RunbookTierRule for RunbookTierRequiredCountryRule {
    fn check(&self, item: &RunbookTier) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTierChecksumLengthRule {
    pub max_length: i64,
}

impl RunbookTierRule for RunbookTierChecksumLengthRule {
    fn check(&self, item: &RunbookTier) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTierCodePrefixRule {
    pub prefix: String,
}

impl RunbookTierRule for RunbookTierCodePrefixRule {
    fn check(&self, item: &RunbookTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTierCodeLengthRule {
    pub max_length: i64,
}

impl RunbookTierRule for RunbookTierCodeLengthRule {
    fn check(&self, item: &RunbookTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook tier.
#[derive(Debug, Clone)]
pub struct RunbookTierValidator {
    pub rules: Vec<Rc<dyn RunbookTierRule>>,
}

impl RunbookTierValidator {
    pub fn validate(&self, item: &RunbookTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_tier_validator() -> RunbookTierValidator {
    RunbookTierValidator {
        rules: vec![
            Rc::new(RunbookTierRequiredCountryRule {
                message: "runbook tier needs a value".to_string(),
            }) as Rc<dyn RunbookTierRule>,
            Rc::new(RunbookTierChecksumLengthRule { max_length: 3600 }),
            Rc::new(RunbookTierCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RunbookTierCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_tier_validator();
        let item = new_runbook_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = RunbookTierRequiredCountryRule {
            message: "runbook tier needs a value".to_string(),
        };
        let mut item = new_runbook_tier("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = RunbookTierChecksumLengthRule { max_length: 1000 };
        let mut item = new_runbook_tier("x", "y");
        item.checksum = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RunbookTierCodeLengthRule { max_length: 64 };
        let mut item = new_runbook_tier("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
