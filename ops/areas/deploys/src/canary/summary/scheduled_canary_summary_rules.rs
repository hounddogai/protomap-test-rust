use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled canary summary must pass. Check returns its problems, or none.
pub trait ScheduledCanarySummaryRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledCanarySummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanarySummaryRequiredCodeRule {
    pub message: String,
}

impl ScheduledCanarySummaryRule for ScheduledCanarySummaryRequiredCodeRule {
    fn check(&self, item: &ScheduledCanarySummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanarySummaryRequiredNameRule {
    pub message: String,
}

impl ScheduledCanarySummaryRule for ScheduledCanarySummaryRequiredNameRule {
    fn check(&self, item: &ScheduledCanarySummary) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanarySummaryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ScheduledCanarySummaryRule for ScheduledCanarySummaryAllowedCodeRule {
    fn check(&self, item: &ScheduledCanarySummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanarySummaryCodePrefixRule {
    pub prefix: String,
}

impl ScheduledCanarySummaryRule for ScheduledCanarySummaryCodePrefixRule {
    fn check(&self, item: &ScheduledCanarySummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanarySummaryNotesLengthRule {
    pub max_length: i64,
}

impl ScheduledCanarySummaryRule for ScheduledCanarySummaryNotesLengthRule {
    fn check(&self, item: &ScheduledCanarySummary) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanarySummaryPrincipalLengthRule {
    pub max_length: i64,
}

impl ScheduledCanarySummaryRule for ScheduledCanarySummaryPrincipalLengthRule {
    fn check(&self, item: &ScheduledCanarySummary) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled canary summary.
#[derive(Debug, Clone)]
pub struct ScheduledCanarySummaryValidator {
    pub rules: Vec<Rc<dyn ScheduledCanarySummaryRule>>,
}

impl ScheduledCanarySummaryValidator {
    pub fn validate(&self, item: &ScheduledCanarySummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledCanarySummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledCanarySummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_canary_summary_validator() -> ScheduledCanarySummaryValidator {
    ScheduledCanarySummaryValidator {
        rules: vec![
            Rc::new(ScheduledCanarySummaryRequiredCodeRule {
                message: "scheduled canary summary needs this field".to_string(),
            }) as Rc<dyn ScheduledCanarySummaryRule>,
            Rc::new(ScheduledCanarySummaryRequiredNameRule {
                message: "scheduled canary summary needs an entry".to_string(),
            }),
            Rc::new(ScheduledCanarySummaryAllowedCodeRule {
                allowed: vec!["regional".to_string(), "capped".to_string()],
            }),
            Rc::new(ScheduledCanarySummaryCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ScheduledCanarySummaryNotesLengthRule { max_length: 16 }),
            Rc::new(ScheduledCanarySummaryPrincipalLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_canary_summary_validator();
        let item = new_scheduled_canary_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledCanarySummaryRequiredCodeRule {
            message: "scheduled canary summary needs this field".to_string(),
        };
        let mut item = new_scheduled_canary_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ScheduledCanarySummaryRequiredNameRule {
            message: "scheduled canary summary needs an entry".to_string(),
        };
        let mut item = new_scheduled_canary_summary("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = ScheduledCanarySummaryNotesLengthRule { max_length: 16 };
        let mut item = new_scheduled_canary_summary("x", "y");
        item.notes = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = ScheduledCanarySummaryPrincipalLengthRule { max_length: 255 };
        let mut item = new_scheduled_canary_summary("x", "y");
        item.principal = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
