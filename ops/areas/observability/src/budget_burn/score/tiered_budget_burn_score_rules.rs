use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered budget burn score must pass. Check returns its problems, or none.
pub trait TieredBudgetBurnScoreRule: std::fmt::Debug {
    fn check(&self, item: &TieredBudgetBurnScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnScoreLabelLengthRule {
    pub max_length: i64,
}

impl TieredBudgetBurnScoreRule for TieredBudgetBurnScoreLabelLengthRule {
    fn check(&self, item: &TieredBudgetBurnScore) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnScoreRequiredCodeRule {
    pub message: String,
}

impl TieredBudgetBurnScoreRule for TieredBudgetBurnScoreRequiredCodeRule {
    fn check(&self, item: &TieredBudgetBurnScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnScoreRequiredAudienceRule {
    pub message: String,
}

impl TieredBudgetBurnScoreRule for TieredBudgetBurnScoreRequiredAudienceRule {
    fn check(&self, item: &TieredBudgetBurnScore) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnScoreNamespaceLengthRule {
    pub max_length: i64,
}

impl TieredBudgetBurnScoreRule for TieredBudgetBurnScoreNamespaceLengthRule {
    fn check(&self, item: &TieredBudgetBurnScore) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered budget burn score.
#[derive(Debug, Clone)]
pub struct TieredBudgetBurnScoreValidator {
    pub rules: Vec<Rc<dyn TieredBudgetBurnScoreRule>>,
}

impl TieredBudgetBurnScoreValidator {
    pub fn validate(&self, item: &TieredBudgetBurnScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredBudgetBurnScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredBudgetBurnScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_budget_burn_score_validator() -> TieredBudgetBurnScoreValidator {
    TieredBudgetBurnScoreValidator {
        rules: vec![
            Rc::new(TieredBudgetBurnScoreLabelLengthRule { max_length: 255 }) as Rc<dyn TieredBudgetBurnScoreRule>,
            Rc::new(TieredBudgetBurnScoreRequiredCodeRule {
                message: "tiered budget burn score needs a value".to_string(),
            }),
            Rc::new(TieredBudgetBurnScoreRequiredAudienceRule {
                message: "tiered budget burn score needs a value".to_string(),
            }),
            Rc::new(TieredBudgetBurnScoreNamespaceLengthRule { max_length: 86400 }),
        ],
    }
}
