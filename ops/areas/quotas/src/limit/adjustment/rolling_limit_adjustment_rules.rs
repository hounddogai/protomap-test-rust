use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling limit adjustment must pass. Check returns its problems, or none.
pub trait RollingLimitAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &RollingLimitAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl RollingLimitAdjustmentRule for RollingLimitAdjustmentCodePrefixRule {
    fn check(&self, item: &RollingLimitAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitAdjustmentAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl RollingLimitAdjustmentRule for RollingLimitAdjustmentAllowedAudienceRule {
    fn check(&self, item: &RollingLimitAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is stale", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitAdjustmentTitleLengthRule {
    pub max_length: i64,
}

impl RollingLimitAdjustmentRule for RollingLimitAdjustmentTitleLengthRule {
    fn check(&self, item: &RollingLimitAdjustment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitAdjustmentRequiredPrincipalRule {
    pub message: String,
}

impl RollingLimitAdjustmentRule for RollingLimitAdjustmentRequiredPrincipalRule {
    fn check(&self, item: &RollingLimitAdjustment) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitAdjustmentChecksumLengthRule {
    pub max_length: i64,
}

impl RollingLimitAdjustmentRule for RollingLimitAdjustmentChecksumLengthRule {
    fn check(&self, item: &RollingLimitAdjustment) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling limit adjustment.
#[derive(Debug, Clone)]
pub struct RollingLimitAdjustmentValidator {
    pub rules: Vec<Rc<dyn RollingLimitAdjustmentRule>>,
}

impl RollingLimitAdjustmentValidator {
    pub fn validate(&self, item: &RollingLimitAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingLimitAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingLimitAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_limit_adjustment_validator() -> RollingLimitAdjustmentValidator {
    RollingLimitAdjustmentValidator {
        rules: vec![
            Rc::new(RollingLimitAdjustmentCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn RollingLimitAdjustmentRule>,
            Rc::new(RollingLimitAdjustmentAllowedAudienceRule {
                allowed: vec!["lenient".to_string(), "capped".to_string()],
            }),
            Rc::new(RollingLimitAdjustmentTitleLengthRule { max_length: 86400 }),
            Rc::new(RollingLimitAdjustmentRequiredPrincipalRule {
                message: "rolling limit adjustment needs a value".to_string(),
            }),
            Rc::new(RollingLimitAdjustmentChecksumLengthRule { max_length: 16 }),
        ],
    }
}
