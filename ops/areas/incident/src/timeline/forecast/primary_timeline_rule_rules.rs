use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary timeline rule must pass. Check returns its problems, or none.
pub trait PrimaryTimelineRuleRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryTimelineRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTimelineRuleAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl PrimaryTimelineRuleRule for PrimaryTimelineRuleAllowedNamespaceRule {
    fn check(&self, item: &PrimaryTimelineRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is below minimum", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTimelineRuleRequiredTitleRule {
    pub message: String,
}

impl PrimaryTimelineRuleRule for PrimaryTimelineRuleRequiredTitleRule {
    fn check(&self, item: &PrimaryTimelineRule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTimelineRuleCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryTimelineRuleRule for PrimaryTimelineRuleCodeLengthRule {
    fn check(&self, item: &PrimaryTimelineRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary timeline rule.
#[derive(Debug, Clone)]
pub struct PrimaryTimelineRuleValidator {
    pub rules: Vec<Rc<dyn PrimaryTimelineRuleRule>>,
}

impl PrimaryTimelineRuleValidator {
    pub fn validate(&self, item: &PrimaryTimelineRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryTimelineRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryTimelineRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_timeline_rule_validator() -> PrimaryTimelineRuleValidator {
    PrimaryTimelineRuleValidator {
        rules: vec![
            Rc::new(PrimaryTimelineRuleAllowedNamespaceRule {
                allowed: vec!["regional".to_string(), "pending".to_string(), "regional".to_string()],
            }) as Rc<dyn PrimaryTimelineRuleRule>,
            Rc::new(PrimaryTimelineRuleRequiredTitleRule {
                message: "primary timeline rule needs an entry".to_string(),
            }),
            Rc::new(PrimaryTimelineRuleCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_timeline_rule_validator();
        let item = new_primary_timeline_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryTimelineRuleRequiredTitleRule {
            message: "primary timeline rule needs an entry".to_string(),
        };
        let mut item = new_primary_timeline_rule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryTimelineRuleCodeLengthRule { max_length: 32 };
        let mut item = new_primary_timeline_rule("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
