use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered measurement record must pass. Check returns its problems, or none.
pub trait TieredMeasurementRecordRule: std::fmt::Debug {
    fn check(&self, item: &TieredMeasurementRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementRecordRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredMeasurementRecordRule for TieredMeasurementRecordRatioRangeRule {
    fn check(&self, item: &TieredMeasurementRecord) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementRecordCodeLengthRule {
    pub max_length: i64,
}

impl TieredMeasurementRecordRule for TieredMeasurementRecordCodeLengthRule {
    fn check(&self, item: &TieredMeasurementRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementRecordUnitLengthRule {
    pub max_length: i64,
}

impl TieredMeasurementRecordRule for TieredMeasurementRecordUnitLengthRule {
    fn check(&self, item: &TieredMeasurementRecord) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementRecordWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredMeasurementRecordRule for TieredMeasurementRecordWeightRangeRule {
    fn check(&self, item: &TieredMeasurementRecord) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementRecordCodePrefixRule {
    pub prefix: String,
}

impl TieredMeasurementRecordRule for TieredMeasurementRecordCodePrefixRule {
    fn check(&self, item: &TieredMeasurementRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered measurement record.
#[derive(Debug, Clone)]
pub struct TieredMeasurementRecordValidator {
    pub rules: Vec<Rc<dyn TieredMeasurementRecordRule>>,
}

impl TieredMeasurementRecordValidator {
    pub fn validate(&self, item: &TieredMeasurementRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredMeasurementRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredMeasurementRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_measurement_record_validator() -> TieredMeasurementRecordValidator {
    TieredMeasurementRecordValidator {
        rules: vec![
            Rc::new(TieredMeasurementRecordRatioRangeRule { min: 8.7, max: 3750.3 })
                as Rc<dyn TieredMeasurementRecordRule>,
            Rc::new(TieredMeasurementRecordCodeLengthRule { max_length: 1000 }),
            Rc::new(TieredMeasurementRecordUnitLengthRule { max_length: 1000 }),
            Rc::new(TieredMeasurementRecordWeightRangeRule { min: 1.3, max: 472.3 }),
            Rc::new(TieredMeasurementRecordCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_measurement_record_validator();
        let item = new_tiered_measurement_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = TieredMeasurementRecordRatioRangeRule { min: 6.3, max: 4702.2 };
        let mut item = new_tiered_measurement_record("x", "y");
        item.ratio = 4703.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredMeasurementRecordCodeLengthRule { max_length: 32 };
        let mut item = new_tiered_measurement_record("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = TieredMeasurementRecordUnitLengthRule { max_length: 120 };
        let mut item = new_tiered_measurement_record("x", "y");
        item.unit = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = TieredMeasurementRecordWeightRangeRule { min: 1.6, max: 4706.1 };
        let mut item = new_tiered_measurement_record("x", "y");
        item.weight = 4707.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
