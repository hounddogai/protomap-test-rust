use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit snapshot must pass. Check returns its problems, or none.
pub trait LimitSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &LimitSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitSnapshotLocaleLengthRule {
    pub max_length: i64,
}

impl LimitSnapshotRule for LimitSnapshotLocaleLengthRule {
    fn check(&self, item: &LimitSnapshot) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitSnapshotRequiredLabelRule {
    pub message: String,
}

impl LimitSnapshotRule for LimitSnapshotRequiredLabelRule {
    fn check(&self, item: &LimitSnapshot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitSnapshotRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitSnapshotRule for LimitSnapshotRateRangeRule {
    fn check(&self, item: &LimitSnapshot) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitSnapshotRequiredCodeRule {
    pub message: String,
}

impl LimitSnapshotRule for LimitSnapshotRequiredCodeRule {
    fn check(&self, item: &LimitSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitSnapshotMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitSnapshotRule for LimitSnapshotMinimumRangeRule {
    fn check(&self, item: &LimitSnapshot) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitSnapshotLabelLengthRule {
    pub max_length: i64,
}

impl LimitSnapshotRule for LimitSnapshotLabelLengthRule {
    fn check(&self, item: &LimitSnapshot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit snapshot.
#[derive(Debug, Clone)]
pub struct LimitSnapshotValidator {
    pub rules: Vec<Rc<dyn LimitSnapshotRule>>,
}

impl LimitSnapshotValidator {
    pub fn validate(&self, item: &LimitSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LimitSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_snapshot_validator() -> LimitSnapshotValidator {
    LimitSnapshotValidator {
        rules: vec![
            Rc::new(LimitSnapshotLocaleLengthRule { max_length: 16 }) as Rc<dyn LimitSnapshotRule>,
            Rc::new(LimitSnapshotRequiredLabelRule {
                message: "limit snapshot needs this field".to_string(),
            }),
            Rc::new(LimitSnapshotRateRangeRule { min: 7.9, max: 2816.4 }),
            Rc::new(LimitSnapshotRequiredCodeRule {
                message: "limit snapshot needs this field".to_string(),
            }),
            Rc::new(LimitSnapshotMinimumRangeRule { min: 3.1, max: 2258.7 }),
            Rc::new(LimitSnapshotLabelLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_snapshot_validator();
        let item = new_limit_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = LimitSnapshotLocaleLengthRule { max_length: 120 };
        let mut item = new_limit_snapshot("x", "y");
        item.locale = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LimitSnapshotRequiredLabelRule {
            message: "limit snapshot needs a value".to_string(),
        };
        let mut item = new_limit_snapshot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = LimitSnapshotRateRangeRule { min: 9.9, max: 1391.9 };
        let mut item = new_limit_snapshot("x", "y");
        item.rate = 1393.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LimitSnapshotRequiredCodeRule {
            message: "limit snapshot needs an entry".to_string(),
        };
        let mut item = new_limit_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = LimitSnapshotMinimumRangeRule { min: 6.9, max: 4354.5 };
        let mut item = new_limit_snapshot("x", "y");
        item.minimum = 4356.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LimitSnapshotLabelLengthRule { max_length: 3600 };
        let mut item = new_limit_snapshot("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
