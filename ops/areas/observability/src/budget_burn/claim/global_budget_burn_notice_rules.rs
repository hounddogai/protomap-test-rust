use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global budget burn notice must pass. Check returns its problems, or none.
pub trait GlobalBudgetBurnNoticeRule: std::fmt::Debug {
    fn check(&self, item: &GlobalBudgetBurnNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnNoticeRequiredTitleRule {
    pub message: String,
}

impl GlobalBudgetBurnNoticeRule for GlobalBudgetBurnNoticeRequiredTitleRule {
    fn check(&self, item: &GlobalBudgetBurnNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnNoticeVisibleGuardRule {
    pub limit: i64,
}

impl GlobalBudgetBurnNoticeRule for GlobalBudgetBurnNoticeVisibleGuardRule {
    fn check(&self, item: &GlobalBudgetBurnNotice) -> Vec<String> {
        if item.visible && (item.port > self.limit) {
            return vec![format!("port {} exceeds {} while visible", item.port, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnNoticeMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalBudgetBurnNoticeRule for GlobalBudgetBurnNoticeMarginRangeRule {
    fn check(&self, item: &GlobalBudgetBurnNotice) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnNoticeTitleLengthRule {
    pub max_length: i64,
}

impl GlobalBudgetBurnNoticeRule for GlobalBudgetBurnNoticeTitleLengthRule {
    fn check(&self, item: &GlobalBudgetBurnNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnNoticeCodePrefixRule {
    pub prefix: String,
}

impl GlobalBudgetBurnNoticeRule for GlobalBudgetBurnNoticeCodePrefixRule {
    fn check(&self, item: &GlobalBudgetBurnNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnNoticeAudienceLengthRule {
    pub max_length: i64,
}

impl GlobalBudgetBurnNoticeRule for GlobalBudgetBurnNoticeAudienceLengthRule {
    fn check(&self, item: &GlobalBudgetBurnNotice) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global budget burn notice.
#[derive(Debug, Clone)]
pub struct GlobalBudgetBurnNoticeValidator {
    pub rules: Vec<Rc<dyn GlobalBudgetBurnNoticeRule>>,
}

impl GlobalBudgetBurnNoticeValidator {
    pub fn validate(&self, item: &GlobalBudgetBurnNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalBudgetBurnNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalBudgetBurnNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_budget_burn_notice_validator() -> GlobalBudgetBurnNoticeValidator {
    GlobalBudgetBurnNoticeValidator {
        rules: vec![
            Rc::new(GlobalBudgetBurnNoticeRequiredTitleRule {
                message: "global budget burn notice needs an entry".to_string(),
            }) as Rc<dyn GlobalBudgetBurnNoticeRule>,
            Rc::new(GlobalBudgetBurnNoticeVisibleGuardRule { limit: 255 }),
            Rc::new(GlobalBudgetBurnNoticeMarginRangeRule { min: 0.4, max: 4152.7 }),
            Rc::new(GlobalBudgetBurnNoticeTitleLengthRule { max_length: 64 }),
            Rc::new(GlobalBudgetBurnNoticeCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(GlobalBudgetBurnNoticeAudienceLengthRule { max_length: 1000 }),
        ],
    }
}
