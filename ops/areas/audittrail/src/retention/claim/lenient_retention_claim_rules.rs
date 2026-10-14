use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient retention claim must pass. Check returns its problems, or none.
pub trait LenientRetentionClaimRule: std::fmt::Debug {
    fn check(&self, item: &LenientRetentionClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionClaimBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientRetentionClaimRule for LenientRetentionClaimBudgetRangeRule {
    fn check(&self, item: &LenientRetentionClaim) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionClaimThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientRetentionClaimRule for LenientRetentionClaimThresholdRangeRule {
    fn check(&self, item: &LenientRetentionClaim) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionClaimAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LenientRetentionClaimRule for LenientRetentionClaimAllowedCodeRule {
    fn check(&self, item: &LenientRetentionClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionClaimRequiredLabelRule {
    pub message: String,
}

impl LenientRetentionClaimRule for LenientRetentionClaimRequiredLabelRule {
    fn check(&self, item: &LenientRetentionClaim) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionClaimGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientRetentionClaimRule for LenientRetentionClaimGraceDaysRangeRule {
    fn check(&self, item: &LenientRetentionClaim) -> Vec<String> {
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

/// Runs every rule of a lenient retention claim.
#[derive(Debug, Clone)]
pub struct LenientRetentionClaimValidator {
    pub rules: Vec<Rc<dyn LenientRetentionClaimRule>>,
}

impl LenientRetentionClaimValidator {
    pub fn validate(&self, item: &LenientRetentionClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientRetentionClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRetentionClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_retention_claim_validator() -> LenientRetentionClaimValidator {
    LenientRetentionClaimValidator {
        rules: vec![
            Rc::new(LenientRetentionClaimBudgetRangeRule { min: 4.6, max: 4873.7 })
                as Rc<dyn LenientRetentionClaimRule>,
            Rc::new(LenientRetentionClaimThresholdRangeRule { min: 4.7, max: 3039.8 }),
            Rc::new(LenientRetentionClaimAllowedCodeRule {
                allowed: vec!["primary".to_string()],
            }),
            Rc::new(LenientRetentionClaimRequiredLabelRule {
                message: "lenient retention claim needs an entry".to_string(),
            }),
            Rc::new(LenientRetentionClaimGraceDaysRangeRule { min: 255, max: 3600 }),
        ],
    }
}
