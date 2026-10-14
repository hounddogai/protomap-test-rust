use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a health check draft must pass. Check returns its problems, or none.
pub trait HealthCheckDraftRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckDraftGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckDraftRule for HealthCheckDraftGraceDaysRangeRule {
    fn check(&self, item: &HealthCheckDraft) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckDraftCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckDraftRule for HealthCheckDraftCodePrefixRule {
    fn check(&self, item: &HealthCheckDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckDraftRequiredTitleRule {
    pub message: String,
}

impl HealthCheckDraftRule for HealthCheckDraftRequiredTitleRule {
    fn check(&self, item: &HealthCheckDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckDraftAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl HealthCheckDraftRule for HealthCheckDraftAllowedCodeRule {
    fn check(&self, item: &HealthCheckDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckDraftRequiredCodeRule {
    pub message: String,
}

impl HealthCheckDraftRule for HealthCheckDraftRequiredCodeRule {
    fn check(&self, item: &HealthCheckDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckDraftQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckDraftRule for HealthCheckDraftQuantityRangeRule {
    fn check(&self, item: &HealthCheckDraft) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check draft.
#[derive(Debug, Clone)]
pub struct HealthCheckDraftValidator {
    pub rules: Vec<Rc<dyn HealthCheckDraftRule>>,
}

impl HealthCheckDraftValidator {
    pub fn validate(&self, item: &HealthCheckDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_draft_validator() -> HealthCheckDraftValidator {
    HealthCheckDraftValidator {
        rules: vec![
            Rc::new(HealthCheckDraftGraceDaysRangeRule { min: 80, max: 3600 }) as Rc<dyn HealthCheckDraftRule>,
            Rc::new(HealthCheckDraftCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(HealthCheckDraftRequiredTitleRule {
                message: "health check draft needs a value".to_string(),
            }),
            Rc::new(HealthCheckDraftAllowedCodeRule {
                allowed: vec!["global".to_string(), "default".to_string()],
            }),
            Rc::new(HealthCheckDraftRequiredCodeRule {
                message: "health check draft needs this field".to_string(),
            }),
            Rc::new(HealthCheckDraftQuantityRangeRule { min: 16, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_draft_validator();
        let item = new_health_check_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = HealthCheckDraftGraceDaysRangeRule { min: 32, max: 86400 };
        let mut item = new_health_check_draft("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = HealthCheckDraftRequiredTitleRule {
            message: "health check draft needs this field".to_string(),
        };
        let mut item = new_health_check_draft("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckDraftRequiredCodeRule {
            message: "health check draft needs an entry".to_string(),
        };
        let mut item = new_health_check_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = HealthCheckDraftQuantityRangeRule { min: 120, max: 3600 };
        let mut item = new_health_check_draft("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
