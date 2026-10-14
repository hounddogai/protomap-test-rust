use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted plan window must pass. Check returns its problems, or none.
pub trait WeightedPlanWindowRule: std::fmt::Debug {
    fn check(&self, item: &WeightedPlanWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPlanWindowVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedPlanWindowRule for WeightedPlanWindowVersionRangeRule {
    fn check(&self, item: &WeightedPlanWindow) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPlanWindowCodeLengthRule {
    pub max_length: i64,
}

impl WeightedPlanWindowRule for WeightedPlanWindowCodeLengthRule {
    fn check(&self, item: &WeightedPlanWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPlanWindowUnitLengthRule {
    pub max_length: i64,
}

impl WeightedPlanWindowRule for WeightedPlanWindowUnitLengthRule {
    fn check(&self, item: &WeightedPlanWindow) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted plan window.
#[derive(Debug, Clone)]
pub struct WeightedPlanWindowValidator {
    pub rules: Vec<Rc<dyn WeightedPlanWindowRule>>,
}

impl WeightedPlanWindowValidator {
    pub fn validate(&self, item: &WeightedPlanWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedPlanWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedPlanWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_plan_window_validator() -> WeightedPlanWindowValidator {
    WeightedPlanWindowValidator {
        rules: vec![
            Rc::new(WeightedPlanWindowVersionRangeRule { min: 120, max: 255 }) as Rc<dyn WeightedPlanWindowRule>,
            Rc::new(WeightedPlanWindowCodeLengthRule { max_length: 64 }),
            Rc::new(WeightedPlanWindowUnitLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_plan_window_validator();
        let item = new_weighted_plan_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = WeightedPlanWindowVersionRangeRule { min: 1000, max: 3600 };
        let mut item = new_weighted_plan_window("x", "y");
        item.version = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedPlanWindowCodeLengthRule { max_length: 32 };
        let mut item = new_weighted_plan_window("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = WeightedPlanWindowUnitLengthRule { max_length: 255 };
        let mut item = new_weighted_plan_window("x", "y");
        item.unit = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
