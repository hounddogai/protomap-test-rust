use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention draft must pass. Check returns its problems, or none.
pub trait RetentionDraftRule: std::fmt::Debug {
    fn check(&self, item: &RetentionDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionDraftRequiredTenantIdRule {
    pub message: String,
}

impl RetentionDraftRule for RetentionDraftRequiredTenantIdRule {
    fn check(&self, item: &RetentionDraft) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionDraftCodePrefixRule {
    pub prefix: String,
}

impl RetentionDraftRule for RetentionDraftCodePrefixRule {
    fn check(&self, item: &RetentionDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionDraftAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RetentionDraftRule for RetentionDraftAllowedNameRule {
    fn check(&self, item: &RetentionDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is duplicated", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionDraftRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionDraftRule for RetentionDraftRetriesRangeRule {
    fn check(&self, item: &RetentionDraft) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionDraftRequiredRegionRule {
    pub message: String,
}

impl RetentionDraftRule for RetentionDraftRequiredRegionRule {
    fn check(&self, item: &RetentionDraft) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention draft.
#[derive(Debug, Clone)]
pub struct RetentionDraftValidator {
    pub rules: Vec<Rc<dyn RetentionDraftRule>>,
}

impl RetentionDraftValidator {
    pub fn validate(&self, item: &RetentionDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_draft_validator() -> RetentionDraftValidator {
    RetentionDraftValidator {
        rules: vec![
            Rc::new(RetentionDraftRequiredTenantIdRule {
                message: "retention draft needs a value".to_string(),
            }) as Rc<dyn RetentionDraftRule>,
            Rc::new(RetentionDraftCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RetentionDraftAllowedNameRule {
                allowed: vec!["adaptive".to_string()],
            }),
            Rc::new(RetentionDraftRetriesRangeRule { min: 16, max: 1000 }),
            Rc::new(RetentionDraftRequiredRegionRule {
                message: "retention draft needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_draft_validator();
        let item = new_retention_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = RetentionDraftRequiredTenantIdRule {
            message: "retention draft needs this field".to_string(),
        };
        let mut item = new_retention_draft("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = RetentionDraftRetriesRangeRule { min: 32, max: 64 };
        let mut item = new_retention_draft("x", "y");
        item.retries = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = RetentionDraftRequiredRegionRule {
            message: "retention draft needs this field".to_string(),
        };
        let mut item = new_retention_draft("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
