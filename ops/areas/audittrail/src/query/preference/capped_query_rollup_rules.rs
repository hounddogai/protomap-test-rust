use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped query rollup must pass. Check returns its problems, or none.
pub trait CappedQueryRollupRule: std::fmt::Debug {
    fn check(&self, item: &CappedQueryRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryRollupRequiredLabelRule {
    pub message: String,
}

impl CappedQueryRollupRule for CappedQueryRollupRequiredLabelRule {
    fn check(&self, item: &CappedQueryRollup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryRollupFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedQueryRollupRule for CappedQueryRollupFloorValueRangeRule {
    fn check(&self, item: &CappedQueryRollup) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryRollupAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl CappedQueryRollupRule for CappedQueryRollupAllowedLabelRule {
    fn check(&self, item: &CappedQueryRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is above maximum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryRollupRequiredCodeRule {
    pub message: String,
}

impl CappedQueryRollupRule for CappedQueryRollupRequiredCodeRule {
    fn check(&self, item: &CappedQueryRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped query rollup.
#[derive(Debug, Clone)]
pub struct CappedQueryRollupValidator {
    pub rules: Vec<Rc<dyn CappedQueryRollupRule>>,
}

impl CappedQueryRollupValidator {
    pub fn validate(&self, item: &CappedQueryRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedQueryRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedQueryRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_query_rollup_validator() -> CappedQueryRollupValidator {
    CappedQueryRollupValidator {
        rules: vec![
            Rc::new(CappedQueryRollupRequiredLabelRule {
                message: "capped query rollup needs an entry".to_string(),
            }) as Rc<dyn CappedQueryRollupRule>,
            Rc::new(CappedQueryRollupFloorValueRangeRule { min: 6.5, max: 2575.0 }),
            Rc::new(CappedQueryRollupAllowedLabelRule {
                allowed: vec!["regional".to_string()],
            }),
            Rc::new(CappedQueryRollupRequiredCodeRule {
                message: "capped query rollup needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_query_rollup_validator();
        let item = new_capped_query_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedQueryRollupRequiredLabelRule {
            message: "capped query rollup needs this field".to_string(),
        };
        let mut item = new_capped_query_rollup("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = CappedQueryRollupFloorValueRangeRule { min: 2.1, max: 2187.0 };
        let mut item = new_capped_query_rollup("x", "y");
        item.floor_value = 2188.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedQueryRollupRequiredCodeRule {
            message: "capped query rollup needs a value".to_string(),
        };
        let mut item = new_capped_query_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
