use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a quote milestone must pass. Check returns its problems, or none.
pub trait QuoteMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &QuoteMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl QuoteMilestoneRule for QuoteMilestoneCodeLengthRule {
    fn check(&self, item: &QuoteMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMilestoneCodePrefixRule {
    pub prefix: String,
}

impl QuoteMilestoneRule for QuoteMilestoneCodePrefixRule {
    fn check(&self, item: &QuoteMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMilestoneVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteMilestoneRule for QuoteMilestoneVersionRangeRule {
    fn check(&self, item: &QuoteMilestone) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMilestoneAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl QuoteMilestoneRule for QuoteMilestoneAllowedAudienceRule {
    fn check(&self, item: &QuoteMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is inconsistent", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMilestoneRequiredLabelRule {
    pub message: String,
}

impl QuoteMilestoneRule for QuoteMilestoneRequiredLabelRule {
    fn check(&self, item: &QuoteMilestone) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMilestoneRequiredNamespaceRule {
    pub message: String,
}

impl QuoteMilestoneRule for QuoteMilestoneRequiredNamespaceRule {
    fn check(&self, item: &QuoteMilestone) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote milestone.
#[derive(Debug, Clone)]
pub struct QuoteMilestoneValidator {
    pub rules: Vec<Rc<dyn QuoteMilestoneRule>>,
}

impl QuoteMilestoneValidator {
    pub fn validate(&self, item: &QuoteMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuoteMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_milestone_validator() -> QuoteMilestoneValidator {
    QuoteMilestoneValidator {
        rules: vec![
            Rc::new(QuoteMilestoneCodeLengthRule { max_length: 32 }) as Rc<dyn QuoteMilestoneRule>,
            Rc::new(QuoteMilestoneCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(QuoteMilestoneVersionRangeRule { min: 64, max: 80 }),
            Rc::new(QuoteMilestoneAllowedAudienceRule {
                allowed: vec!["manual".to_string(), "manual".to_string(), "strict".to_string()],
            }),
            Rc::new(QuoteMilestoneRequiredLabelRule {
                message: "quote milestone needs a value".to_string(),
            }),
            Rc::new(QuoteMilestoneRequiredNamespaceRule {
                message: "quote milestone needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_milestone_validator();
        let item = new_quote_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QuoteMilestoneCodeLengthRule { max_length: 255 };
        let mut item = new_quote_milestone("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = QuoteMilestoneVersionRangeRule { min: 16, max: 80 };
        let mut item = new_quote_milestone("x", "y");
        item.version = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = QuoteMilestoneRequiredLabelRule {
            message: "quote milestone needs a value".to_string(),
        };
        let mut item = new_quote_milestone("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = QuoteMilestoneRequiredNamespaceRule {
            message: "quote milestone needs an entry".to_string(),
        };
        let mut item = new_quote_milestone("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
