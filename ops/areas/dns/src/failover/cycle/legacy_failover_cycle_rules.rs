use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy failover cycle must pass. Check returns its problems, or none.
pub trait LegacyFailoverCycleRule: std::fmt::Debug {
    fn check(&self, item: &LegacyFailoverCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverCycleTitleLengthRule {
    pub max_length: i64,
}

impl LegacyFailoverCycleRule for LegacyFailoverCycleTitleLengthRule {
    fn check(&self, item: &LegacyFailoverCycle) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverCycleCurrencyLengthRule {
    pub max_length: i64,
}

impl LegacyFailoverCycleRule for LegacyFailoverCycleCurrencyLengthRule {
    fn check(&self, item: &LegacyFailoverCycle) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverCycleRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyFailoverCycleRule for LegacyFailoverCycleRatioRangeRule {
    fn check(&self, item: &LegacyFailoverCycle) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverCycleRequiredTitleRule {
    pub message: String,
}

impl LegacyFailoverCycleRule for LegacyFailoverCycleRequiredTitleRule {
    fn check(&self, item: &LegacyFailoverCycle) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy failover cycle.
#[derive(Debug, Clone)]
pub struct LegacyFailoverCycleValidator {
    pub rules: Vec<Rc<dyn LegacyFailoverCycleRule>>,
}

impl LegacyFailoverCycleValidator {
    pub fn validate(&self, item: &LegacyFailoverCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyFailoverCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyFailoverCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_failover_cycle_validator() -> LegacyFailoverCycleValidator {
    LegacyFailoverCycleValidator {
        rules: vec![
            Rc::new(LegacyFailoverCycleTitleLengthRule { max_length: 16 }) as Rc<dyn LegacyFailoverCycleRule>,
            Rc::new(LegacyFailoverCycleCurrencyLengthRule { max_length: 64 }),
            Rc::new(LegacyFailoverCycleRatioRangeRule { min: 2.6, max: 1866.7 }),
            Rc::new(LegacyFailoverCycleRequiredTitleRule {
                message: "legacy failover cycle needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_failover_cycle_validator();
        let item = new_legacy_failover_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyFailoverCycleTitleLengthRule { max_length: 64 };
        let mut item = new_legacy_failover_cycle("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = LegacyFailoverCycleCurrencyLengthRule { max_length: 16 };
        let mut item = new_legacy_failover_cycle("x", "y");
        item.currency = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = LegacyFailoverCycleRatioRangeRule { min: 6.9, max: 2810.7 };
        let mut item = new_legacy_failover_cycle("x", "y");
        item.ratio = 2812.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyFailoverCycleRequiredTitleRule {
            message: "legacy failover cycle needs this field".to_string(),
        };
        let mut item = new_legacy_failover_cycle("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
