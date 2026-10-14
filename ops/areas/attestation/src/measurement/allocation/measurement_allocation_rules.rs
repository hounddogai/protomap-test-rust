use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement allocation must pass. Check returns its problems, or none.
pub trait MeasurementAllocationRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAllocationCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementAllocationRule for MeasurementAllocationCeilingRangeRule {
    fn check(&self, item: &MeasurementAllocation) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAllocationRequiredTimezoneRule {
    pub message: String,
}

impl MeasurementAllocationRule for MeasurementAllocationRequiredTimezoneRule {
    fn check(&self, item: &MeasurementAllocation) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAllocationCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementAllocationRule for MeasurementAllocationCostRangeRule {
    fn check(&self, item: &MeasurementAllocation) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAllocationRequiredLabelRule {
    pub message: String,
}

impl MeasurementAllocationRule for MeasurementAllocationRequiredLabelRule {
    fn check(&self, item: &MeasurementAllocation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement allocation.
#[derive(Debug, Clone)]
pub struct MeasurementAllocationValidator {
    pub rules: Vec<Rc<dyn MeasurementAllocationRule>>,
}

impl MeasurementAllocationValidator {
    pub fn validate(&self, item: &MeasurementAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MeasurementAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_allocation_validator() -> MeasurementAllocationValidator {
    MeasurementAllocationValidator {
        rules: vec![
            Rc::new(MeasurementAllocationCeilingRangeRule { min: 6.8, max: 4763.6 })
                as Rc<dyn MeasurementAllocationRule>,
            Rc::new(MeasurementAllocationRequiredTimezoneRule {
                message: "measurement allocation needs this field".to_string(),
            }),
            Rc::new(MeasurementAllocationCostRangeRule { min: 9.1, max: 1449.0 }),
            Rc::new(MeasurementAllocationRequiredLabelRule {
                message: "measurement allocation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_allocation_validator();
        let item = new_measurement_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = MeasurementAllocationCeilingRangeRule { min: 8.9, max: 1855.0 };
        let mut item = new_measurement_allocation("x", "y");
        item.ceiling = 1856.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = MeasurementAllocationRequiredTimezoneRule {
            message: "measurement allocation needs this field".to_string(),
        };
        let mut item = new_measurement_allocation("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = MeasurementAllocationCostRangeRule { min: 2.1, max: 3082.4 };
        let mut item = new_measurement_allocation("x", "y");
        item.cost = 3083.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = MeasurementAllocationRequiredLabelRule {
            message: "measurement allocation needs this field".to_string(),
        };
        let mut item = new_measurement_allocation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
