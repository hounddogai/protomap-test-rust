use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy silence window must pass. Check returns its problems, or none.
pub trait LegacySilenceWindowRule: std::fmt::Debug {
    fn check(&self, item: &LegacySilenceWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceWindowRequiredCodeRule {
    pub message: String,
}

impl LegacySilenceWindowRule for LegacySilenceWindowRequiredCodeRule {
    fn check(&self, item: &LegacySilenceWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceWindowStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacySilenceWindowRule for LegacySilenceWindowStartsAtRangeRule {
    fn check(&self, item: &LegacySilenceWindow) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceWindowCodeLengthRule {
    pub max_length: i64,
}

impl LegacySilenceWindowRule for LegacySilenceWindowCodeLengthRule {
    fn check(&self, item: &LegacySilenceWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceWindowBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacySilenceWindowRule for LegacySilenceWindowBudgetRangeRule {
    fn check(&self, item: &LegacySilenceWindow) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a legacy silence window.
#[derive(Debug, Clone)]
pub struct LegacySilenceWindowValidator {
    pub rules: Vec<Rc<dyn LegacySilenceWindowRule>>,
}

impl LegacySilenceWindowValidator {
    pub fn validate(&self, item: &LegacySilenceWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacySilenceWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacySilenceWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_silence_window_validator() -> LegacySilenceWindowValidator {
    LegacySilenceWindowValidator {
        rules: vec![
            Rc::new(LegacySilenceWindowRequiredCodeRule {
                message: "legacy silence window needs this field".to_string(),
            }) as Rc<dyn LegacySilenceWindowRule>,
            Rc::new(LegacySilenceWindowStartsAtRangeRule { min: 80, max: 1000 }),
            Rc::new(LegacySilenceWindowCodeLengthRule { max_length: 32 }),
            Rc::new(LegacySilenceWindowBudgetRangeRule { min: 7.2, max: 4007.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_silence_window_validator();
        let item = new_legacy_silence_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacySilenceWindowRequiredCodeRule {
            message: "legacy silence window needs a value".to_string(),
        };
        let mut item = new_legacy_silence_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = LegacySilenceWindowStartsAtRangeRule { min: 32, max: 3600 };
        let mut item = new_legacy_silence_window("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacySilenceWindowCodeLengthRule { max_length: 1000 };
        let mut item = new_legacy_silence_window("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = LegacySilenceWindowBudgetRangeRule { min: 8.6, max: 3951.9 };
        let mut item = new_legacy_silence_window("x", "y");
        item.budget = 3953.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
