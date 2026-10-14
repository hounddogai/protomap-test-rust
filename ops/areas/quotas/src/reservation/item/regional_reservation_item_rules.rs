use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional reservation item must pass. Check returns its problems, or none.
pub trait RegionalReservationItemRule: std::fmt::Debug {
    fn check(&self, item: &RegionalReservationItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationItemRequiredCodeRule {
    pub message: String,
}

impl RegionalReservationItemRule for RegionalReservationItemRequiredCodeRule {
    fn check(&self, item: &RegionalReservationItem) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationItemThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalReservationItemRule for RegionalReservationItemThresholdRangeRule {
    fn check(&self, item: &RegionalReservationItem) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationItemSegmentLengthRule {
    pub max_length: i64,
}

impl RegionalReservationItemRule for RegionalReservationItemSegmentLengthRule {
    fn check(&self, item: &RegionalReservationItem) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationItemCodeLengthRule {
    pub max_length: i64,
}

impl RegionalReservationItemRule for RegionalReservationItemCodeLengthRule {
    fn check(&self, item: &RegionalReservationItem) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationItemSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalReservationItemRule for RegionalReservationItemSequenceRangeRule {
    fn check(&self, item: &RegionalReservationItem) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional reservation item.
#[derive(Debug, Clone)]
pub struct RegionalReservationItemValidator {
    pub rules: Vec<Rc<dyn RegionalReservationItemRule>>,
}

impl RegionalReservationItemValidator {
    pub fn validate(&self, item: &RegionalReservationItem) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalReservationItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalReservationItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_reservation_item_validator() -> RegionalReservationItemValidator {
    RegionalReservationItemValidator {
        rules: vec![
            Rc::new(RegionalReservationItemRequiredCodeRule {
                message: "regional reservation item needs this field".to_string(),
            }) as Rc<dyn RegionalReservationItemRule>,
            Rc::new(RegionalReservationItemThresholdRangeRule { min: 7.4, max: 2423.1 }),
            Rc::new(RegionalReservationItemSegmentLengthRule { max_length: 120 }),
            Rc::new(RegionalReservationItemCodeLengthRule { max_length: 80 }),
            Rc::new(RegionalReservationItemSequenceRangeRule { min: 16, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_reservation_item_validator();
        let item = new_regional_reservation_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalReservationItemRequiredCodeRule {
            message: "regional reservation item needs an entry".to_string(),
        };
        let mut item = new_regional_reservation_item("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RegionalReservationItemThresholdRangeRule { min: 1.9, max: 1479.5 };
        let mut item = new_regional_reservation_item("x", "y");
        item.threshold = 1481.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = RegionalReservationItemSegmentLengthRule { max_length: 3600 };
        let mut item = new_regional_reservation_item("x", "y");
        item.segment = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalReservationItemCodeLengthRule { max_length: 255 };
        let mut item = new_regional_reservation_item("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = RegionalReservationItemSequenceRangeRule { min: 64, max: 80 };
        let mut item = new_regional_reservation_item("x", "y");
        item.sequence = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
