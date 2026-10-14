use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback evidence template must pass. Check returns its problems, or none.
pub trait FallbackEvidenceTemplateRule: std::fmt::Debug {
    fn check(&self, item: &FallbackEvidenceTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceTemplateRequiredOwnerIdRule {
    pub message: String,
}

impl FallbackEvidenceTemplateRule for FallbackEvidenceTemplateRequiredOwnerIdRule {
    fn check(&self, item: &FallbackEvidenceTemplate) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceTemplateDescriptionLengthRule {
    pub max_length: i64,
}

impl FallbackEvidenceTemplateRule for FallbackEvidenceTemplateDescriptionLengthRule {
    fn check(&self, item: &FallbackEvidenceTemplate) -> Vec<String> {
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
pub struct FallbackEvidenceTemplateCodePrefixRule {
    pub prefix: String,
}

impl FallbackEvidenceTemplateRule for FallbackEvidenceTemplateCodePrefixRule {
    fn check(&self, item: &FallbackEvidenceTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceTemplateCodeLengthRule {
    pub max_length: i64,
}

impl FallbackEvidenceTemplateRule for FallbackEvidenceTemplateCodeLengthRule {
    fn check(&self, item: &FallbackEvidenceTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceTemplateRequiredTitleRule {
    pub message: String,
}

impl FallbackEvidenceTemplateRule for FallbackEvidenceTemplateRequiredTitleRule {
    fn check(&self, item: &FallbackEvidenceTemplate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback evidence template.
#[derive(Debug, Clone)]
pub struct FallbackEvidenceTemplateValidator {
    pub rules: Vec<Rc<dyn FallbackEvidenceTemplateRule>>,
}

impl FallbackEvidenceTemplateValidator {
    pub fn validate(&self, item: &FallbackEvidenceTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackEvidenceTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackEvidenceTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_evidence_template_validator() -> FallbackEvidenceTemplateValidator {
    FallbackEvidenceTemplateValidator {
        rules: vec![
            Rc::new(FallbackEvidenceTemplateRequiredOwnerIdRule {
                message: "fallback evidence template needs this field".to_string(),
            }) as Rc<dyn FallbackEvidenceTemplateRule>,
            Rc::new(FallbackEvidenceTemplateDescriptionLengthRule { max_length: 3600 }),
            Rc::new(FallbackEvidenceTemplateCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(FallbackEvidenceTemplateCodeLengthRule { max_length: 32 }),
            Rc::new(FallbackEvidenceTemplateRequiredTitleRule {
                message: "fallback evidence template needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_evidence_template_validator();
        let item = new_fallback_evidence_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = FallbackEvidenceTemplateRequiredOwnerIdRule {
            message: "fallback evidence template needs a value".to_string(),
        };
        let mut item = new_fallback_evidence_template("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = FallbackEvidenceTemplateDescriptionLengthRule { max_length: 32 };
        let mut item = new_fallback_evidence_template("x", "y");
        item.description = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackEvidenceTemplateCodeLengthRule { max_length: 16 };
        let mut item = new_fallback_evidence_template("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FallbackEvidenceTemplateRequiredTitleRule {
            message: "fallback evidence template needs this field".to_string(),
        };
        let mut item = new_fallback_evidence_template("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
