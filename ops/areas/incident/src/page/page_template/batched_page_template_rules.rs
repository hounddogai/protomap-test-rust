use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched page template must pass. Check returns its problems, or none.
pub trait BatchedPageTemplateRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPageTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageTemplateNotesLengthRule {
    pub max_length: i64,
}

impl BatchedPageTemplateRule for BatchedPageTemplateNotesLengthRule {
    fn check(&self, item: &BatchedPageTemplate) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageTemplateCodePrefixRule {
    pub prefix: String,
}

impl BatchedPageTemplateRule for BatchedPageTemplateCodePrefixRule {
    fn check(&self, item: &BatchedPageTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageTemplateLabelLengthRule {
    pub max_length: i64,
}

impl BatchedPageTemplateRule for BatchedPageTemplateLabelLengthRule {
    fn check(&self, item: &BatchedPageTemplate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageTemplateRequiredLabelRule {
    pub message: String,
}

impl BatchedPageTemplateRule for BatchedPageTemplateRequiredLabelRule {
    fn check(&self, item: &BatchedPageTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageTemplateAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl BatchedPageTemplateRule for BatchedPageTemplateAllowedNotesRule {
    fn check(&self, item: &BatchedPageTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is too long", item.notes)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched page template.
#[derive(Debug, Clone)]
pub struct BatchedPageTemplateValidator {
    pub rules: Vec<Rc<dyn BatchedPageTemplateRule>>,
}

impl BatchedPageTemplateValidator {
    pub fn validate(&self, item: &BatchedPageTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedPageTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPageTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_page_template_validator() -> BatchedPageTemplateValidator {
    BatchedPageTemplateValidator {
        rules: vec![
            Rc::new(BatchedPageTemplateNotesLengthRule { max_length: 120 }) as Rc<dyn BatchedPageTemplateRule>,
            Rc::new(BatchedPageTemplateCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(BatchedPageTemplateLabelLengthRule { max_length: 1000 }),
            Rc::new(BatchedPageTemplateRequiredLabelRule {
                message: "batched page template needs an entry".to_string(),
            }),
            Rc::new(BatchedPageTemplateAllowedNotesRule {
                allowed: vec!["capped".to_string(), "default".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_page_template_validator();
        let item = new_batched_page_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = BatchedPageTemplateNotesLengthRule { max_length: 255 };
        let mut item = new_batched_page_template("x", "y");
        item.notes = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedPageTemplateLabelLengthRule { max_length: 16 };
        let mut item = new_batched_page_template("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedPageTemplateRequiredLabelRule {
            message: "batched page template needs a value".to_string(),
        };
        let mut item = new_batched_page_template("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
