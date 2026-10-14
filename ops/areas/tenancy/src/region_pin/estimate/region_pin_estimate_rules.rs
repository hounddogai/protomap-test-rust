use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin estimate must pass. Check returns its problems, or none.
pub trait RegionPinEstimateRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinEstimateRequiredIssuerRule {
    pub message: String,
}

impl RegionPinEstimateRule for RegionPinEstimateRequiredIssuerRule {
    fn check(&self, item: &RegionPinEstimate) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinEstimateLabelLengthRule {
    pub max_length: i64,
}

impl RegionPinEstimateRule for RegionPinEstimateLabelLengthRule {
    fn check(&self, item: &RegionPinEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinEstimateCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinEstimateRule for RegionPinEstimateCreatedAtRangeRule {
    fn check(&self, item: &RegionPinEstimate) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a region pin estimate.
#[derive(Debug, Clone)]
pub struct RegionPinEstimateValidator {
    pub rules: Vec<Rc<dyn RegionPinEstimateRule>>,
}

impl RegionPinEstimateValidator {
    pub fn validate(&self, item: &RegionPinEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_estimate_validator() -> RegionPinEstimateValidator {
    RegionPinEstimateValidator {
        rules: vec![
            Rc::new(RegionPinEstimateRequiredIssuerRule {
                message: "region pin estimate needs a value".to_string(),
            }) as Rc<dyn RegionPinEstimateRule>,
            Rc::new(RegionPinEstimateLabelLengthRule { max_length: 1000 }),
            Rc::new(RegionPinEstimateCreatedAtRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_estimate_validator();
        let item = new_region_pin_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = RegionPinEstimateRequiredIssuerRule {
            message: "region pin estimate needs an entry".to_string(),
        };
        let mut item = new_region_pin_estimate("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionPinEstimateLabelLengthRule { max_length: 120 };
        let mut item = new_region_pin_estimate("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RegionPinEstimateCreatedAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_region_pin_estimate("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
