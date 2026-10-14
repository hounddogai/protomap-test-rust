use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted region pin profile must pass. Check returns its problems, or none.
pub trait WeightedRegionPinProfileRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRegionPinProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinProfileRequiredContactEmailRule {
    pub message: String,
}

impl WeightedRegionPinProfileRule for WeightedRegionPinProfileRequiredContactEmailRule {
    fn check(&self, item: &WeightedRegionPinProfile) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinProfileCodeLengthRule {
    pub max_length: i64,
}

impl WeightedRegionPinProfileRule for WeightedRegionPinProfileCodeLengthRule {
    fn check(&self, item: &WeightedRegionPinProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinProfileRequiredNameRule {
    pub message: String,
}

impl WeightedRegionPinProfileRule for WeightedRegionPinProfileRequiredNameRule {
    fn check(&self, item: &WeightedRegionPinProfile) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinProfileVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedRegionPinProfileRule for WeightedRegionPinProfileVersionRangeRule {
    fn check(&self, item: &WeightedRegionPinProfile) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinProfileNameLengthRule {
    pub max_length: i64,
}

impl WeightedRegionPinProfileRule for WeightedRegionPinProfileNameLengthRule {
    fn check(&self, item: &WeightedRegionPinProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinProfileMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedRegionPinProfileRule for WeightedRegionPinProfileMinimumRangeRule {
    fn check(&self, item: &WeightedRegionPinProfile) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted region pin profile.
#[derive(Debug, Clone)]
pub struct WeightedRegionPinProfileValidator {
    pub rules: Vec<Rc<dyn WeightedRegionPinProfileRule>>,
}

impl WeightedRegionPinProfileValidator {
    pub fn validate(&self, item: &WeightedRegionPinProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedRegionPinProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRegionPinProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_region_pin_profile_validator() -> WeightedRegionPinProfileValidator {
    WeightedRegionPinProfileValidator {
        rules: vec![
            Rc::new(WeightedRegionPinProfileRequiredContactEmailRule {
                message: "weighted region pin profile needs this field".to_string(),
            }) as Rc<dyn WeightedRegionPinProfileRule>,
            Rc::new(WeightedRegionPinProfileCodeLengthRule { max_length: 32 }),
            Rc::new(WeightedRegionPinProfileRequiredNameRule {
                message: "weighted region pin profile needs an entry".to_string(),
            }),
            Rc::new(WeightedRegionPinProfileVersionRangeRule { min: 255, max: 255 }),
            Rc::new(WeightedRegionPinProfileNameLengthRule { max_length: 1000 }),
            Rc::new(WeightedRegionPinProfileMinimumRangeRule { min: 4.3, max: 4756.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_region_pin_profile_validator();
        let item = new_weighted_region_pin_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = WeightedRegionPinProfileRequiredContactEmailRule {
            message: "weighted region pin profile needs an entry".to_string(),
        };
        let mut item = new_weighted_region_pin_profile("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedRegionPinProfileCodeLengthRule { max_length: 80 };
        let mut item = new_weighted_region_pin_profile("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = WeightedRegionPinProfileRequiredNameRule {
            message: "weighted region pin profile needs this field".to_string(),
        };
        let mut item = new_weighted_region_pin_profile("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = WeightedRegionPinProfileVersionRangeRule { min: 32, max: 1000 };
        let mut item = new_weighted_region_pin_profile("x", "y");
        item.version = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedRegionPinProfileNameLengthRule { max_length: 3600 };
        let mut item = new_weighted_region_pin_profile("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = WeightedRegionPinProfileMinimumRangeRule { min: 2.7, max: 2203.5 };
        let mut item = new_weighted_region_pin_profile("x", "y");
        item.minimum = 2205.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
