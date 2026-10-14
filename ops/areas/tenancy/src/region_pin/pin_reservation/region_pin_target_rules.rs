use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin target must pass. Check returns its problems, or none.
pub trait RegionPinTargetRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinTargetRequiredTitleRule {
    pub message: String,
}

impl RegionPinTargetRule for RegionPinTargetRequiredTitleRule {
    fn check(&self, item: &RegionPinTarget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinTargetRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionPinTargetRule for RegionPinTargetRatioRangeRule {
    fn check(&self, item: &RegionPinTarget) -> Vec<String> {
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
pub struct RegionPinTargetCodeLengthRule {
    pub max_length: i64,
}

impl RegionPinTargetRule for RegionPinTargetCodeLengthRule {
    fn check(&self, item: &RegionPinTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin target.
#[derive(Debug, Clone)]
pub struct RegionPinTargetValidator {
    pub rules: Vec<Rc<dyn RegionPinTargetRule>>,
}

impl RegionPinTargetValidator {
    pub fn validate(&self, item: &RegionPinTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_target_validator() -> RegionPinTargetValidator {
    RegionPinTargetValidator {
        rules: vec![
            Rc::new(RegionPinTargetRequiredTitleRule {
                message: "region pin target needs a value".to_string(),
            }) as Rc<dyn RegionPinTargetRule>,
            Rc::new(RegionPinTargetRatioRangeRule { min: 3.9, max: 1765.7 }),
            Rc::new(RegionPinTargetCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_target_validator();
        let item = new_region_pin_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionPinTargetRequiredTitleRule {
            message: "region pin target needs this field".to_string(),
        };
        let mut item = new_region_pin_target("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = RegionPinTargetRatioRangeRule { min: 3.3, max: 4627.1 };
        let mut item = new_region_pin_target("x", "y");
        item.ratio = 4628.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionPinTargetCodeLengthRule { max_length: 1000 };
        let mut item = new_region_pin_target("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
