use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive identity schedule must pass. Check returns its problems, or none.
pub trait AdaptiveIdentityScheduleRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveIdentitySchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityScheduleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveIdentityScheduleRule for AdaptiveIdentityScheduleAllowedCodeRule {
    fn check(&self, item: &AdaptiveIdentitySchedule) -> Vec<String> {
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
pub struct AdaptiveIdentityScheduleApprovedGuardRule {
    pub limit: i64,
}

impl AdaptiveIdentityScheduleRule for AdaptiveIdentityScheduleApprovedGuardRule {
    fn check(&self, item: &AdaptiveIdentitySchedule) -> Vec<String> {
        if item.approved && (item.sequence > self.limit) {
            return vec![format!(
                "sequence {} exceeds {} while approved",
                item.sequence, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityScheduleRequiredTitleRule {
    pub message: String,
}

impl AdaptiveIdentityScheduleRule for AdaptiveIdentityScheduleRequiredTitleRule {
    fn check(&self, item: &AdaptiveIdentitySchedule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityScheduleRequiredIssuerRule {
    pub message: String,
}

impl AdaptiveIdentityScheduleRule for AdaptiveIdentityScheduleRequiredIssuerRule {
    fn check(&self, item: &AdaptiveIdentitySchedule) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityScheduleCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveIdentityScheduleRule for AdaptiveIdentityScheduleCodeLengthRule {
    fn check(&self, item: &AdaptiveIdentitySchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityScheduleTitleLengthRule {
    pub max_length: i64,
}

impl AdaptiveIdentityScheduleRule for AdaptiveIdentityScheduleTitleLengthRule {
    fn check(&self, item: &AdaptiveIdentitySchedule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive identity schedule.
#[derive(Debug, Clone)]
pub struct AdaptiveIdentityScheduleValidator {
    pub rules: Vec<Rc<dyn AdaptiveIdentityScheduleRule>>,
}

impl AdaptiveIdentityScheduleValidator {
    pub fn validate(&self, item: &AdaptiveIdentitySchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveIdentitySchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveIdentitySchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_identity_schedule_validator() -> AdaptiveIdentityScheduleValidator {
    AdaptiveIdentityScheduleValidator {
        rules: vec![
            Rc::new(AdaptiveIdentityScheduleAllowedCodeRule {
                allowed: vec!["regional".to_string()],
            }) as Rc<dyn AdaptiveIdentityScheduleRule>,
            Rc::new(AdaptiveIdentityScheduleApprovedGuardRule { limit: 32 }),
            Rc::new(AdaptiveIdentityScheduleRequiredTitleRule {
                message: "adaptive identity schedule needs an entry".to_string(),
            }),
            Rc::new(AdaptiveIdentityScheduleRequiredIssuerRule {
                message: "adaptive identity schedule needs a value".to_string(),
            }),
            Rc::new(AdaptiveIdentityScheduleCodeLengthRule { max_length: 64 }),
            Rc::new(AdaptiveIdentityScheduleTitleLengthRule { max_length: 3600 }),
        ],
    }
}
