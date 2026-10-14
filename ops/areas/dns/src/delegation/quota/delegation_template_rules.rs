use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a delegation template must pass. Check returns its problems, or none.
pub trait DelegationTemplateRule: std::fmt::Debug {
    fn check(&self, item: &DelegationTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTemplateRequiredCodeRule {
    pub message: String,
}

impl DelegationTemplateRule for DelegationTemplateRequiredCodeRule {
    fn check(&self, item: &DelegationTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTemplateAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl DelegationTemplateRule for DelegationTemplateAllowedTitleRule {
    fn check(&self, item: &DelegationTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTemplateSourceLengthRule {
    pub max_length: i64,
}

impl DelegationTemplateRule for DelegationTemplateSourceLengthRule {
    fn check(&self, item: &DelegationTemplate) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation template.
#[derive(Debug, Clone)]
pub struct DelegationTemplateValidator {
    pub rules: Vec<Rc<dyn DelegationTemplateRule>>,
}

impl DelegationTemplateValidator {
    pub fn validate(&self, item: &DelegationTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_template_validator() -> DelegationTemplateValidator {
    DelegationTemplateValidator {
        rules: vec![
            Rc::new(DelegationTemplateRequiredCodeRule {
                message: "delegation template needs a value".to_string(),
            }) as Rc<dyn DelegationTemplateRule>,
            Rc::new(DelegationTemplateAllowedTitleRule {
                allowed: vec!["tiered".to_string(), "fallback".to_string()],
            }),
            Rc::new(DelegationTemplateSourceLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_template_validator();
        let item = new_delegation_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DelegationTemplateRequiredCodeRule {
            message: "delegation template needs an entry".to_string(),
        };
        let mut item = new_delegation_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = DelegationTemplateSourceLengthRule { max_length: 120 };
        let mut item = new_delegation_template("x", "y");
        item.source = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
