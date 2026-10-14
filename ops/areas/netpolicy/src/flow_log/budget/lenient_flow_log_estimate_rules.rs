use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient flow log estimate must pass. Check returns its problems, or none.
pub trait LenientFlowLogEstimateRule: std::fmt::Debug {
    fn check(&self, item: &LenientFlowLogEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEstimateAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl LenientFlowLogEstimateRule for LenientFlowLogEstimateAllowedScopeRule {
    fn check(&self, item: &LenientFlowLogEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is inconsistent", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEstimatePinnedGuardRule {
    pub limit: f64,
}

impl LenientFlowLogEstimateRule for LenientFlowLogEstimatePinnedGuardRule {
    fn check(&self, item: &LenientFlowLogEstimate) -> Vec<String> {
        if item.pinned && (item.minimum > self.limit) {
            return vec![format!("minimum {} exceeds {} while pinned", item.minimum, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEstimateCodeLengthRule {
    pub max_length: i64,
}

impl LenientFlowLogEstimateRule for LenientFlowLogEstimateCodeLengthRule {
    fn check(&self, item: &LenientFlowLogEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEstimateQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientFlowLogEstimateRule for LenientFlowLogEstimateQuantityRangeRule {
    fn check(&self, item: &LenientFlowLogEstimate) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEstimateRequiredLabelRule {
    pub message: String,
}

impl LenientFlowLogEstimateRule for LenientFlowLogEstimateRequiredLabelRule {
    fn check(&self, item: &LenientFlowLogEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEstimateExternalRefLengthRule {
    pub max_length: i64,
}

impl LenientFlowLogEstimateRule for LenientFlowLogEstimateExternalRefLengthRule {
    fn check(&self, item: &LenientFlowLogEstimate) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient flow log estimate.
#[derive(Debug, Clone)]
pub struct LenientFlowLogEstimateValidator {
    pub rules: Vec<Rc<dyn LenientFlowLogEstimateRule>>,
}

impl LenientFlowLogEstimateValidator {
    pub fn validate(&self, item: &LenientFlowLogEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientFlowLogEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientFlowLogEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_flow_log_estimate_validator() -> LenientFlowLogEstimateValidator {
    LenientFlowLogEstimateValidator {
        rules: vec![
            Rc::new(LenientFlowLogEstimateAllowedScopeRule {
                allowed: vec!["manual".to_string()],
            }) as Rc<dyn LenientFlowLogEstimateRule>,
            Rc::new(LenientFlowLogEstimatePinnedGuardRule { limit: 5.5 }),
            Rc::new(LenientFlowLogEstimateCodeLengthRule { max_length: 3600 }),
            Rc::new(LenientFlowLogEstimateQuantityRangeRule { min: 80, max: 255 }),
            Rc::new(LenientFlowLogEstimateRequiredLabelRule {
                message: "lenient flow log estimate needs a value".to_string(),
            }),
            Rc::new(LenientFlowLogEstimateExternalRefLengthRule { max_length: 1000 }),
        ],
    }
}
