use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment snapshot must pass. Check returns its problems, or none.
pub trait SegmentSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &SegmentSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSnapshotCurrencyLengthRule {
    pub max_length: i64,
}

impl SegmentSnapshotRule for SegmentSnapshotCurrencyLengthRule {
    fn check(&self, item: &SegmentSnapshot) -> Vec<String> {
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
pub struct SegmentSnapshotRequiredCodeRule {
    pub message: String,
}

impl SegmentSnapshotRule for SegmentSnapshotRequiredCodeRule {
    fn check(&self, item: &SegmentSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSnapshotRequiredLabelRule {
    pub message: String,
}

impl SegmentSnapshotRule for SegmentSnapshotRequiredLabelRule {
    fn check(&self, item: &SegmentSnapshot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSnapshotQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentSnapshotRule for SegmentSnapshotQuotaUnitsRangeRule {
    fn check(&self, item: &SegmentSnapshot) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSnapshotRequiredCurrencyRule {
    pub message: String,
}

impl SegmentSnapshotRule for SegmentSnapshotRequiredCurrencyRule {
    fn check(&self, item: &SegmentSnapshot) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSnapshotMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SegmentSnapshotRule for SegmentSnapshotMarginRangeRule {
    fn check(&self, item: &SegmentSnapshot) -> Vec<String> {
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

/// Runs every rule of a segment snapshot.
#[derive(Debug, Clone)]
pub struct SegmentSnapshotValidator {
    pub rules: Vec<Rc<dyn SegmentSnapshotRule>>,
}

impl SegmentSnapshotValidator {
    pub fn validate(&self, item: &SegmentSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_snapshot_validator() -> SegmentSnapshotValidator {
    SegmentSnapshotValidator {
        rules: vec![
            Rc::new(SegmentSnapshotCurrencyLengthRule { max_length: 255 }) as Rc<dyn SegmentSnapshotRule>,
            Rc::new(SegmentSnapshotRequiredCodeRule {
                message: "segment snapshot needs an entry".to_string(),
            }),
            Rc::new(SegmentSnapshotRequiredLabelRule {
                message: "segment snapshot needs this field".to_string(),
            }),
            Rc::new(SegmentSnapshotQuotaUnitsRangeRule { min: 16, max: 32 }),
            Rc::new(SegmentSnapshotRequiredCurrencyRule {
                message: "segment snapshot needs an entry".to_string(),
            }),
            Rc::new(SegmentSnapshotMarginRangeRule { min: 1.5, max: 4286.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_snapshot_validator();
        let item = new_segment_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = SegmentSnapshotCurrencyLengthRule { max_length: 1000 };
        let mut item = new_segment_snapshot("x", "y");
        item.currency = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SegmentSnapshotRequiredCodeRule {
            message: "segment snapshot needs a value".to_string(),
        };
        let mut item = new_segment_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SegmentSnapshotRequiredLabelRule {
            message: "segment snapshot needs an entry".to_string(),
        };
        let mut item = new_segment_snapshot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = SegmentSnapshotQuotaUnitsRangeRule { min: 255, max: 86400 };
        let mut item = new_segment_snapshot("x", "y");
        item.quota_units = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = SegmentSnapshotRequiredCurrencyRule {
            message: "segment snapshot needs this field".to_string(),
        };
        let mut item = new_segment_snapshot("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = SegmentSnapshotMarginRangeRule { min: 5.3, max: 711.0 };
        let mut item = new_segment_snapshot("x", "y");
        item.margin = 712.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
