use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback token notice must pass. Check returns its problems, or none.
pub trait FallbackTokenNoticeRule: std::fmt::Debug {
    fn check(&self, item: &FallbackTokenNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTokenNoticeRequiredScopeRule {
    pub message: String,
}

impl FallbackTokenNoticeRule for FallbackTokenNoticeRequiredScopeRule {
    fn check(&self, item: &FallbackTokenNotice) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTokenNoticeCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackTokenNoticeRule for FallbackTokenNoticeCostRangeRule {
    fn check(&self, item: &FallbackTokenNotice) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTokenNoticeRequiredDescriptionRule {
    pub message: String,
}

impl FallbackTokenNoticeRule for FallbackTokenNoticeRequiredDescriptionRule {
    fn check(&self, item: &FallbackTokenNotice) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTokenNoticeCodePrefixRule {
    pub prefix: String,
}

impl FallbackTokenNoticeRule for FallbackTokenNoticeCodePrefixRule {
    fn check(&self, item: &FallbackTokenNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTokenNoticeAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl FallbackTokenNoticeRule for FallbackTokenNoticeAllowedDescriptionRule {
    fn check(&self, item: &FallbackTokenNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is expired", item.description)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback token notice.
#[derive(Debug, Clone)]
pub struct FallbackTokenNoticeValidator {
    pub rules: Vec<Rc<dyn FallbackTokenNoticeRule>>,
}

impl FallbackTokenNoticeValidator {
    pub fn validate(&self, item: &FallbackTokenNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackTokenNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackTokenNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_token_notice_validator() -> FallbackTokenNoticeValidator {
    FallbackTokenNoticeValidator {
        rules: vec![
            Rc::new(FallbackTokenNoticeRequiredScopeRule {
                message: "fallback token notice needs an entry".to_string(),
            }) as Rc<dyn FallbackTokenNoticeRule>,
            Rc::new(FallbackTokenNoticeCostRangeRule { min: 7.0, max: 3824.2 }),
            Rc::new(FallbackTokenNoticeRequiredDescriptionRule {
                message: "fallback token notice needs a value".to_string(),
            }),
            Rc::new(FallbackTokenNoticeCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(FallbackTokenNoticeAllowedDescriptionRule {
                allowed: vec!["strict".to_string(), "batched".to_string(), "legacy".to_string()],
            }),
        ],
    }
}
