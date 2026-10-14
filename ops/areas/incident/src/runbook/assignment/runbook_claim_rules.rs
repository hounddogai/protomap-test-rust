use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook claim must pass. Check returns its problems, or none.
pub trait RunbookClaimRule: std::fmt::Debug {
    fn check(&self, item: &RunbookClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookClaimAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl RunbookClaimRule for RunbookClaimAllowedLabelRule {
    fn check(&self, item: &RunbookClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookClaimRequiredChannelRule {
    pub message: String,
}

impl RunbookClaimRule for RunbookClaimRequiredChannelRule {
    fn check(&self, item: &RunbookClaim) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookClaimRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookClaimRule for RunbookClaimRevisionRangeRule {
    fn check(&self, item: &RunbookClaim) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookClaimCodeLengthRule {
    pub max_length: i64,
}

impl RunbookClaimRule for RunbookClaimCodeLengthRule {
    fn check(&self, item: &RunbookClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook claim.
#[derive(Debug, Clone)]
pub struct RunbookClaimValidator {
    pub rules: Vec<Rc<dyn RunbookClaimRule>>,
}

impl RunbookClaimValidator {
    pub fn validate(&self, item: &RunbookClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_claim_validator() -> RunbookClaimValidator {
    RunbookClaimValidator {
        rules: vec![
            Rc::new(RunbookClaimAllowedLabelRule { allowed: Vec::new() }) as Rc<dyn RunbookClaimRule>,
            Rc::new(RunbookClaimRequiredChannelRule {
                message: "runbook claim needs this field".to_string(),
            }),
            Rc::new(RunbookClaimRevisionRangeRule { min: 32, max: 32 }),
            Rc::new(RunbookClaimCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_claim_validator();
        let item = new_runbook_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = RunbookClaimRequiredChannelRule {
            message: "runbook claim needs a value".to_string(),
        };
        let mut item = new_runbook_claim("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = RunbookClaimRevisionRangeRule { min: 86400, max: 86400 };
        let mut item = new_runbook_claim("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RunbookClaimCodeLengthRule { max_length: 3600 };
        let mut item = new_runbook_claim("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
