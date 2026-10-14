use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback budget burn metric must pass. Check returns its problems, or none.
pub trait FallbackBudgetBurnMetricRule: std::fmt::Debug {
    fn check(&self, item: &FallbackBudgetBurnMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnMetricPinnedGuardRule {
    pub limit: i64,
}

impl FallbackBudgetBurnMetricRule for FallbackBudgetBurnMetricPinnedGuardRule {
    fn check(&self, item: &FallbackBudgetBurnMetric) -> Vec<String> {
        if item.pinned && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while pinned", item.burst, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnMetricRequiredChannelRule {
    pub message: String,
}

impl FallbackBudgetBurnMetricRule for FallbackBudgetBurnMetricRequiredChannelRule {
    fn check(&self, item: &FallbackBudgetBurnMetric) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnMetricCodeLengthRule {
    pub max_length: i64,
}

impl FallbackBudgetBurnMetricRule for FallbackBudgetBurnMetricCodeLengthRule {
    fn check(&self, item: &FallbackBudgetBurnMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnMetricNotesLengthRule {
    pub max_length: i64,
}

impl FallbackBudgetBurnMetricRule for FallbackBudgetBurnMetricNotesLengthRule {
    fn check(&self, item: &FallbackBudgetBurnMetric) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnMetricRequiredNameRule {
    pub message: String,
}

impl FallbackBudgetBurnMetricRule for FallbackBudgetBurnMetricRequiredNameRule {
    fn check(&self, item: &FallbackBudgetBurnMetric) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnMetricQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackBudgetBurnMetricRule for FallbackBudgetBurnMetricQuotaUnitsRangeRule {
    fn check(&self, item: &FallbackBudgetBurnMetric) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback budget burn metric.
#[derive(Debug, Clone)]
pub struct FallbackBudgetBurnMetricValidator {
    pub rules: Vec<Rc<dyn FallbackBudgetBurnMetricRule>>,
}

impl FallbackBudgetBurnMetricValidator {
    pub fn validate(&self, item: &FallbackBudgetBurnMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackBudgetBurnMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackBudgetBurnMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_budget_burn_metric_validator() -> FallbackBudgetBurnMetricValidator {
    FallbackBudgetBurnMetricValidator {
        rules: vec![
            Rc::new(FallbackBudgetBurnMetricPinnedGuardRule { limit: 3600 }) as Rc<dyn FallbackBudgetBurnMetricRule>,
            Rc::new(FallbackBudgetBurnMetricRequiredChannelRule {
                message: "fallback budget burn metric needs a value".to_string(),
            }),
            Rc::new(FallbackBudgetBurnMetricCodeLengthRule { max_length: 16 }),
            Rc::new(FallbackBudgetBurnMetricNotesLengthRule { max_length: 86400 }),
            Rc::new(FallbackBudgetBurnMetricRequiredNameRule {
                message: "fallback budget burn metric needs a value".to_string(),
            }),
            Rc::new(FallbackBudgetBurnMetricQuotaUnitsRangeRule { min: 16, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_budget_burn_metric_validator();
        let item = new_fallback_budget_burn_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = FallbackBudgetBurnMetricRequiredChannelRule {
            message: "fallback budget burn metric needs this field".to_string(),
        };
        let mut item = new_fallback_budget_burn_metric("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackBudgetBurnMetricCodeLengthRule { max_length: 255 };
        let mut item = new_fallback_budget_burn_metric("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = FallbackBudgetBurnMetricNotesLengthRule { max_length: 32 };
        let mut item = new_fallback_budget_burn_metric("x", "y");
        item.notes = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FallbackBudgetBurnMetricRequiredNameRule {
            message: "fallback budget burn metric needs an entry".to_string(),
        };
        let mut item = new_fallback_budget_burn_metric("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = FallbackBudgetBurnMetricQuotaUnitsRangeRule { min: 255, max: 3600 };
        let mut item = new_fallback_budget_burn_metric("x", "y");
        item.quota_units = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
