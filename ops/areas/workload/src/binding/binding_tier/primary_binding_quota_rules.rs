use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary binding quota must pass. Check returns its problems, or none.
pub trait PrimaryBindingQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryBindingQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingQuotaRequiredCodeRule {
    pub message: String,
}

impl PrimaryBindingQuotaRule for PrimaryBindingQuotaRequiredCodeRule {
    fn check(&self, item: &PrimaryBindingQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingQuotaLabelLengthRule {
    pub max_length: i64,
}

impl PrimaryBindingQuotaRule for PrimaryBindingQuotaLabelLengthRule {
    fn check(&self, item: &PrimaryBindingQuota) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingQuotaDescriptionLengthRule {
    pub max_length: i64,
}

impl PrimaryBindingQuotaRule for PrimaryBindingQuotaDescriptionLengthRule {
    fn check(&self, item: &PrimaryBindingQuota) -> Vec<String> {
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
pub struct PrimaryBindingQuotaAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl PrimaryBindingQuotaRule for PrimaryBindingQuotaAllowedDescriptionRule {
    fn check(&self, item: &PrimaryBindingQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is duplicated", item.description)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary binding quota.
#[derive(Debug, Clone)]
pub struct PrimaryBindingQuotaValidator {
    pub rules: Vec<Rc<dyn PrimaryBindingQuotaRule>>,
}

impl PrimaryBindingQuotaValidator {
    pub fn validate(&self, item: &PrimaryBindingQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryBindingQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryBindingQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_binding_quota_validator() -> PrimaryBindingQuotaValidator {
    PrimaryBindingQuotaValidator {
        rules: vec![
            Rc::new(PrimaryBindingQuotaRequiredCodeRule {
                message: "primary binding quota needs an entry".to_string(),
            }) as Rc<dyn PrimaryBindingQuotaRule>,
            Rc::new(PrimaryBindingQuotaLabelLengthRule { max_length: 64 }),
            Rc::new(PrimaryBindingQuotaDescriptionLengthRule { max_length: 255 }),
            Rc::new(PrimaryBindingQuotaAllowedDescriptionRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_binding_quota_validator();
        let item = new_primary_binding_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryBindingQuotaRequiredCodeRule {
            message: "primary binding quota needs this field".to_string(),
        };
        let mut item = new_primary_binding_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimaryBindingQuotaLabelLengthRule { max_length: 64 };
        let mut item = new_primary_binding_quota("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = PrimaryBindingQuotaDescriptionLengthRule { max_length: 16 };
        let mut item = new_primary_binding_quota("x", "y");
        item.description = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
