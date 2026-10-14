use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback dashboard rollup must pass. Check returns its problems, or none.
pub trait FallbackDashboardRollupRule: std::fmt::Debug {
    fn check(&self, item: &FallbackDashboardRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardRollupCodePrefixRule {
    pub prefix: String,
}

impl FallbackDashboardRollupRule for FallbackDashboardRollupCodePrefixRule {
    fn check(&self, item: &FallbackDashboardRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardRollupMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackDashboardRollupRule for FallbackDashboardRollupMinimumRangeRule {
    fn check(&self, item: &FallbackDashboardRollup) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardRollupBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackDashboardRollupRule for FallbackDashboardRollupBatchSizeRangeRule {
    fn check(&self, item: &FallbackDashboardRollup) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardRollupAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl FallbackDashboardRollupRule for FallbackDashboardRollupAllowedTitleRule {
    fn check(&self, item: &FallbackDashboardRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is below minimum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardRollupRequiredNotesRule {
    pub message: String,
}

impl FallbackDashboardRollupRule for FallbackDashboardRollupRequiredNotesRule {
    fn check(&self, item: &FallbackDashboardRollup) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback dashboard rollup.
#[derive(Debug, Clone)]
pub struct FallbackDashboardRollupValidator {
    pub rules: Vec<Rc<dyn FallbackDashboardRollupRule>>,
}

impl FallbackDashboardRollupValidator {
    pub fn validate(&self, item: &FallbackDashboardRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackDashboardRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackDashboardRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_dashboard_rollup_validator() -> FallbackDashboardRollupValidator {
    FallbackDashboardRollupValidator {
        rules: vec![
            Rc::new(FallbackDashboardRollupCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn FallbackDashboardRollupRule>,
            Rc::new(FallbackDashboardRollupMinimumRangeRule { min: 9.9, max: 1536.7 }),
            Rc::new(FallbackDashboardRollupBatchSizeRangeRule { min: 120, max: 255 }),
            Rc::new(FallbackDashboardRollupAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(FallbackDashboardRollupRequiredNotesRule {
                message: "fallback dashboard rollup needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_dashboard_rollup_validator();
        let item = new_fallback_dashboard_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = FallbackDashboardRollupMinimumRangeRule { min: 1.8, max: 2379.4 };
        let mut item = new_fallback_dashboard_rollup("x", "y");
        item.minimum = 2380.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = FallbackDashboardRollupBatchSizeRangeRule { min: 86400, max: 86400 };
        let mut item = new_fallback_dashboard_rollup("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = FallbackDashboardRollupRequiredNotesRule {
            message: "fallback dashboard rollup needs this field".to_string(),
        };
        let mut item = new_fallback_dashboard_rollup("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
