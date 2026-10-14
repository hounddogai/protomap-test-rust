use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive quote rollup must pass. Check returns its problems, or none.
pub trait AdaptiveQuoteRollupRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveQuoteRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteRollupRequiredNameRule {
    pub message: String,
}

impl AdaptiveQuoteRollupRule for AdaptiveQuoteRollupRequiredNameRule {
    fn check(&self, item: &AdaptiveQuoteRollup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteRollupLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveQuoteRollupRule for AdaptiveQuoteRollupLabelLengthRule {
    fn check(&self, item: &AdaptiveQuoteRollup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteRollupTenantIdLengthRule {
    pub max_length: i64,
}

impl AdaptiveQuoteRollupRule for AdaptiveQuoteRollupTenantIdLengthRule {
    fn check(&self, item: &AdaptiveQuoteRollup) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteRollupRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveQuoteRollupRule for AdaptiveQuoteRollupRotationDaysRangeRule {
    fn check(&self, item: &AdaptiveQuoteRollup) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteRollupAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl AdaptiveQuoteRollupRule for AdaptiveQuoteRollupAllowedClusterRule {
    fn check(&self, item: &AdaptiveQuoteRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is above maximum", item.cluster)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteRollupBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveQuoteRollupRule for AdaptiveQuoteRollupBurstRangeRule {
    fn check(&self, item: &AdaptiveQuoteRollup) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive quote rollup.
#[derive(Debug, Clone)]
pub struct AdaptiveQuoteRollupValidator {
    pub rules: Vec<Rc<dyn AdaptiveQuoteRollupRule>>,
}

impl AdaptiveQuoteRollupValidator {
    pub fn validate(&self, item: &AdaptiveQuoteRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveQuoteRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveQuoteRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_quote_rollup_validator() -> AdaptiveQuoteRollupValidator {
    AdaptiveQuoteRollupValidator {
        rules: vec![
            Rc::new(AdaptiveQuoteRollupRequiredNameRule {
                message: "adaptive quote rollup needs a value".to_string(),
            }) as Rc<dyn AdaptiveQuoteRollupRule>,
            Rc::new(AdaptiveQuoteRollupLabelLengthRule { max_length: 3600 }),
            Rc::new(AdaptiveQuoteRollupTenantIdLengthRule { max_length: 120 }),
            Rc::new(AdaptiveQuoteRollupRotationDaysRangeRule { min: 64, max: 3600 }),
            Rc::new(AdaptiveQuoteRollupAllowedClusterRule { allowed: Vec::new() }),
            Rc::new(AdaptiveQuoteRollupBurstRangeRule { min: 64, max: 86400 }),
        ],
    }
}
