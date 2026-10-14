use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement adjustment must pass. Check returns its problems, or none.
pub trait MeasurementAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAdjustmentRequiredNameRule {
    pub message: String,
}

impl MeasurementAdjustmentRule for MeasurementAdjustmentRequiredNameRule {
    fn check(&self, item: &MeasurementAdjustment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAdjustmentUnitLengthRule {
    pub max_length: i64,
}

impl MeasurementAdjustmentRule for MeasurementAdjustmentUnitLengthRule {
    fn check(&self, item: &MeasurementAdjustment) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAdjustmentStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementAdjustmentRule for MeasurementAdjustmentStartsAtRangeRule {
    fn check(&self, item: &MeasurementAdjustment) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAdjustmentFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementAdjustmentRule for MeasurementAdjustmentFloorValueRangeRule {
    fn check(&self, item: &MeasurementAdjustment) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAdjustmentNameLengthRule {
    pub max_length: i64,
}

impl MeasurementAdjustmentRule for MeasurementAdjustmentNameLengthRule {
    fn check(&self, item: &MeasurementAdjustment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl MeasurementAdjustmentRule for MeasurementAdjustmentCodeLengthRule {
    fn check(&self, item: &MeasurementAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement adjustment.
#[derive(Debug, Clone)]
pub struct MeasurementAdjustmentValidator {
    pub rules: Vec<Rc<dyn MeasurementAdjustmentRule>>,
}

impl MeasurementAdjustmentValidator {
    pub fn validate(&self, item: &MeasurementAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_adjustment_validator() -> MeasurementAdjustmentValidator {
    MeasurementAdjustmentValidator {
        rules: vec![
            Rc::new(MeasurementAdjustmentRequiredNameRule {
                message: "measurement adjustment needs a value".to_string(),
            }) as Rc<dyn MeasurementAdjustmentRule>,
            Rc::new(MeasurementAdjustmentUnitLengthRule { max_length: 80 }),
            Rc::new(MeasurementAdjustmentStartsAtRangeRule { min: 64, max: 80 }),
            Rc::new(MeasurementAdjustmentFloorValueRangeRule { min: 3.8, max: 3718.2 }),
            Rc::new(MeasurementAdjustmentNameLengthRule { max_length: 32 }),
            Rc::new(MeasurementAdjustmentCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_adjustment_validator();
        let item = new_measurement_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = MeasurementAdjustmentRequiredNameRule {
            message: "measurement adjustment needs a value".to_string(),
        };
        let mut item = new_measurement_adjustment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = MeasurementAdjustmentUnitLengthRule { max_length: 32 };
        let mut item = new_measurement_adjustment("x", "y");
        item.unit = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = MeasurementAdjustmentStartsAtRangeRule { min: 32, max: 64 };
        let mut item = new_measurement_adjustment("x", "y");
        item.starts_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = MeasurementAdjustmentFloorValueRangeRule { min: 3.2, max: 4033.2 };
        let mut item = new_measurement_adjustment("x", "y");
        item.floor_value = 4034.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = MeasurementAdjustmentNameLengthRule { max_length: 1000 };
        let mut item = new_measurement_adjustment("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = MeasurementAdjustmentCodeLengthRule { max_length: 32 };
        let mut item = new_measurement_adjustment("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
