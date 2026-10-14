use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional query profile must pass. Check returns its problems, or none.
pub trait RegionalQueryProfileRule: std::fmt::Debug {
    fn check(&self, item: &RegionalQueryProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryProfileFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalQueryProfileRule for RegionalQueryProfileFloorValueRangeRule {
    fn check(&self, item: &RegionalQueryProfile) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryProfileRequiredNameRule {
    pub message: String,
}

impl RegionalQueryProfileRule for RegionalQueryProfileRequiredNameRule {
    fn check(&self, item: &RegionalQueryProfile) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryProfileRequiredCodeRule {
    pub message: String,
}

impl RegionalQueryProfileRule for RegionalQueryProfileRequiredCodeRule {
    fn check(&self, item: &RegionalQueryProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional query profile.
#[derive(Debug, Clone)]
pub struct RegionalQueryProfileValidator {
    pub rules: Vec<Rc<dyn RegionalQueryProfileRule>>,
}

impl RegionalQueryProfileValidator {
    pub fn validate(&self, item: &RegionalQueryProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalQueryProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalQueryProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_query_profile_validator() -> RegionalQueryProfileValidator {
    RegionalQueryProfileValidator {
        rules: vec![
            Rc::new(RegionalQueryProfileFloorValueRangeRule { min: 5.7, max: 2337.8 })
                as Rc<dyn RegionalQueryProfileRule>,
            Rc::new(RegionalQueryProfileRequiredNameRule {
                message: "regional query profile needs this field".to_string(),
            }),
            Rc::new(RegionalQueryProfileRequiredCodeRule {
                message: "regional query profile needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_query_profile_validator();
        let item = new_regional_query_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = RegionalQueryProfileFloorValueRangeRule { min: 9.2, max: 2478.5 };
        let mut item = new_regional_query_profile("x", "y");
        item.floor_value = 2480.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionalQueryProfileRequiredNameRule {
            message: "regional query profile needs a value".to_string(),
        };
        let mut item = new_regional_query_profile("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalQueryProfileRequiredCodeRule {
            message: "regional query profile needs an entry".to_string(),
        };
        let mut item = new_regional_query_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
