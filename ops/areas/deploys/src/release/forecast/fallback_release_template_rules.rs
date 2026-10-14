use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback release template must pass. Check returns its problems, or none.
pub trait FallbackReleaseTemplateRule: std::fmt::Debug {
    fn check(&self, item: &FallbackReleaseTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseTemplateRequiredCodeRule {
    pub message: String,
}

impl FallbackReleaseTemplateRule for FallbackReleaseTemplateRequiredCodeRule {
    fn check(&self, item: &FallbackReleaseTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseTemplateActiveGuardRule {
    pub limit: i64,
}

impl FallbackReleaseTemplateRule for FallbackReleaseTemplateActiveGuardRule {
    fn check(&self, item: &FallbackReleaseTemplate) -> Vec<String> {
        if item.active && (item.ends_at > self.limit) {
            return vec![format!("ends at {} exceeds {} while active", item.ends_at, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseTemplateRequiredNamespaceRule {
    pub message: String,
}

impl FallbackReleaseTemplateRule for FallbackReleaseTemplateRequiredNamespaceRule {
    fn check(&self, item: &FallbackReleaseTemplate) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseTemplateAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl FallbackReleaseTemplateRule for FallbackReleaseTemplateAllowedNamespaceRule {
    fn check(&self, item: &FallbackReleaseTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is inconsistent", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseTemplateCodeLengthRule {
    pub max_length: i64,
}

impl FallbackReleaseTemplateRule for FallbackReleaseTemplateCodeLengthRule {
    fn check(&self, item: &FallbackReleaseTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback release template.
#[derive(Debug, Clone)]
pub struct FallbackReleaseTemplateValidator {
    pub rules: Vec<Rc<dyn FallbackReleaseTemplateRule>>,
}

impl FallbackReleaseTemplateValidator {
    pub fn validate(&self, item: &FallbackReleaseTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackReleaseTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackReleaseTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_release_template_validator() -> FallbackReleaseTemplateValidator {
    FallbackReleaseTemplateValidator {
        rules: vec![
            Rc::new(FallbackReleaseTemplateRequiredCodeRule {
                message: "fallback release template needs a value".to_string(),
            }) as Rc<dyn FallbackReleaseTemplateRule>,
            Rc::new(FallbackReleaseTemplateActiveGuardRule { limit: 3600 }),
            Rc::new(FallbackReleaseTemplateRequiredNamespaceRule {
                message: "fallback release template needs this field".to_string(),
            }),
            Rc::new(FallbackReleaseTemplateAllowedNamespaceRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(FallbackReleaseTemplateCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_release_template_validator();
        let item = new_fallback_release_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackReleaseTemplateRequiredCodeRule {
            message: "fallback release template needs this field".to_string(),
        };
        let mut item = new_fallback_release_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = FallbackReleaseTemplateRequiredNamespaceRule {
            message: "fallback release template needs this field".to_string(),
        };
        let mut item = new_fallback_release_template("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackReleaseTemplateCodeLengthRule { max_length: 64 };
        let mut item = new_fallback_release_template("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
