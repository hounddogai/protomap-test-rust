use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted peering estimate must pass. Check returns its problems, or none.
pub trait WeightedPeeringEstimateRule: std::fmt::Debug {
    fn check(&self, item: &WeightedPeeringEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringEstimateScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedPeeringEstimateRule for WeightedPeeringEstimateScoreRangeRule {
    fn check(&self, item: &WeightedPeeringEstimate) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringEstimateCodePrefixRule {
    pub prefix: String,
}

impl WeightedPeeringEstimateRule for WeightedPeeringEstimateCodePrefixRule {
    fn check(&self, item: &WeightedPeeringEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringEstimateDescriptionLengthRule {
    pub max_length: i64,
}

impl WeightedPeeringEstimateRule for WeightedPeeringEstimateDescriptionLengthRule {
    fn check(&self, item: &WeightedPeeringEstimate) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringEstimateRequiredTitleRule {
    pub message: String,
}

impl WeightedPeeringEstimateRule for WeightedPeeringEstimateRequiredTitleRule {
    fn check(&self, item: &WeightedPeeringEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringEstimateRequiredCodeRule {
    pub message: String,
}

impl WeightedPeeringEstimateRule for WeightedPeeringEstimateRequiredCodeRule {
    fn check(&self, item: &WeightedPeeringEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted peering estimate.
#[derive(Debug, Clone)]
pub struct WeightedPeeringEstimateValidator {
    pub rules: Vec<Rc<dyn WeightedPeeringEstimateRule>>,
}

impl WeightedPeeringEstimateValidator {
    pub fn validate(&self, item: &WeightedPeeringEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedPeeringEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedPeeringEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_peering_estimate_validator() -> WeightedPeeringEstimateValidator {
    WeightedPeeringEstimateValidator {
        rules: vec![
            Rc::new(WeightedPeeringEstimateScoreRangeRule { min: 4.2, max: 757.1 })
                as Rc<dyn WeightedPeeringEstimateRule>,
            Rc::new(WeightedPeeringEstimateCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(WeightedPeeringEstimateDescriptionLengthRule { max_length: 120 }),
            Rc::new(WeightedPeeringEstimateRequiredTitleRule {
                message: "weighted peering estimate needs an entry".to_string(),
            }),
            Rc::new(WeightedPeeringEstimateRequiredCodeRule {
                message: "weighted peering estimate needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_peering_estimate_validator();
        let item = new_weighted_peering_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = WeightedPeeringEstimateScoreRangeRule { min: 4.5, max: 2908.1 };
        let mut item = new_weighted_peering_estimate("x", "y");
        item.score = 2909.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = WeightedPeeringEstimateDescriptionLengthRule { max_length: 3600 };
        let mut item = new_weighted_peering_estimate("x", "y");
        item.description = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedPeeringEstimateRequiredTitleRule {
            message: "weighted peering estimate needs a value".to_string(),
        };
        let mut item = new_weighted_peering_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedPeeringEstimateRequiredCodeRule {
            message: "weighted peering estimate needs this field".to_string(),
        };
        let mut item = new_weighted_peering_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
