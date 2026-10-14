use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered rollout template must pass. Check returns its problems, or none.
pub trait TieredRolloutTemplateRule: std::fmt::Debug {
    fn check(&self, item: &TieredRolloutTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutTemplateAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl TieredRolloutTemplateRule for TieredRolloutTemplateAllowedTitleRule {
    fn check(&self, item: &TieredRolloutTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutTemplateTitleLengthRule {
    pub max_length: i64,
}

impl TieredRolloutTemplateRule for TieredRolloutTemplateTitleLengthRule {
    fn check(&self, item: &TieredRolloutTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutTemplateCodePrefixRule {
    pub prefix: String,
}

impl TieredRolloutTemplateRule for TieredRolloutTemplateCodePrefixRule {
    fn check(&self, item: &TieredRolloutTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutTemplateCodeLengthRule {
    pub max_length: i64,
}

impl TieredRolloutTemplateRule for TieredRolloutTemplateCodeLengthRule {
    fn check(&self, item: &TieredRolloutTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutTemplateRequiredTitleRule {
    pub message: String,
}

impl TieredRolloutTemplateRule for TieredRolloutTemplateRequiredTitleRule {
    fn check(&self, item: &TieredRolloutTemplate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered rollout template.
#[derive(Debug, Clone)]
pub struct TieredRolloutTemplateValidator {
    pub rules: Vec<Rc<dyn TieredRolloutTemplateRule>>,
}

impl TieredRolloutTemplateValidator {
    pub fn validate(&self, item: &TieredRolloutTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredRolloutTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredRolloutTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_rollout_template_validator() -> TieredRolloutTemplateValidator {
    TieredRolloutTemplateValidator {
        rules: vec![
            Rc::new(TieredRolloutTemplateAllowedTitleRule {
                allowed: vec!["pending".to_string(), "weighted".to_string(), "manual".to_string()],
            }) as Rc<dyn TieredRolloutTemplateRule>,
            Rc::new(TieredRolloutTemplateTitleLengthRule { max_length: 64 }),
            Rc::new(TieredRolloutTemplateCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(TieredRolloutTemplateCodeLengthRule { max_length: 255 }),
            Rc::new(TieredRolloutTemplateRequiredTitleRule {
                message: "tiered rollout template needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_rollout_template_validator();
        let item = new_tiered_rollout_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredRolloutTemplateTitleLengthRule { max_length: 16 };
        let mut item = new_tiered_rollout_template("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredRolloutTemplateCodeLengthRule { max_length: 3600 };
        let mut item = new_tiered_rollout_template("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredRolloutTemplateRequiredTitleRule {
            message: "tiered rollout template needs this field".to_string(),
        };
        let mut item = new_tiered_rollout_template("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
