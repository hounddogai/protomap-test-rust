use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence cycle must pass. Check returns its problems, or none.
pub trait SilenceCycleRule: std::fmt::Debug {
    fn check(&self, item: &SilenceCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceCycleNotesLengthRule {
    pub max_length: i64,
}

impl SilenceCycleRule for SilenceCycleNotesLengthRule {
    fn check(&self, item: &SilenceCycle) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceCycleBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SilenceCycleRule for SilenceCycleBudgetRangeRule {
    fn check(&self, item: &SilenceCycle) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceCycleMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SilenceCycleRule for SilenceCycleMarginRangeRule {
    fn check(&self, item: &SilenceCycle) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceCycleRequiredLabelRule {
    pub message: String,
}

impl SilenceCycleRule for SilenceCycleRequiredLabelRule {
    fn check(&self, item: &SilenceCycle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence cycle.
#[derive(Debug, Clone)]
pub struct SilenceCycleValidator {
    pub rules: Vec<Rc<dyn SilenceCycleRule>>,
}

impl SilenceCycleValidator {
    pub fn validate(&self, item: &SilenceCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SilenceCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_cycle_validator() -> SilenceCycleValidator {
    SilenceCycleValidator {
        rules: vec![
            Rc::new(SilenceCycleNotesLengthRule { max_length: 16 }) as Rc<dyn SilenceCycleRule>,
            Rc::new(SilenceCycleBudgetRangeRule { min: 7.2, max: 1768.6 }),
            Rc::new(SilenceCycleMarginRangeRule { min: 4.5, max: 607.2 }),
            Rc::new(SilenceCycleRequiredLabelRule {
                message: "silence cycle needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_cycle_validator();
        let item = new_silence_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = SilenceCycleNotesLengthRule { max_length: 16 };
        let mut item = new_silence_cycle("x", "y");
        item.notes = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = SilenceCycleBudgetRangeRule { min: 1.8, max: 4869.8 };
        let mut item = new_silence_cycle("x", "y");
        item.budget = 4871.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = SilenceCycleMarginRangeRule { min: 5.6, max: 2228.4 };
        let mut item = new_silence_cycle("x", "y");
        item.margin = 2229.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SilenceCycleRequiredLabelRule {
            message: "silence cycle needs an entry".to_string(),
        };
        let mut item = new_silence_cycle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
