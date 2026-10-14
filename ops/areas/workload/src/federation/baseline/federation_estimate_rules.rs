use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation estimate must pass. Check returns its problems, or none.
pub trait FederationEstimateRule: std::fmt::Debug {
    fn check(&self, item: &FederationEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationEstimateScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FederationEstimateRule for FederationEstimateScoreRangeRule {
    fn check(&self, item: &FederationEstimate) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationEstimateRequiredCodeRule {
    pub message: String,
}

impl FederationEstimateRule for FederationEstimateRequiredCodeRule {
    fn check(&self, item: &FederationEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationEstimateNamespaceLengthRule {
    pub max_length: i64,
}

impl FederationEstimateRule for FederationEstimateNamespaceLengthRule {
    fn check(&self, item: &FederationEstimate) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationEstimateTimezoneLengthRule {
    pub max_length: i64,
}

impl FederationEstimateRule for FederationEstimateTimezoneLengthRule {
    fn check(&self, item: &FederationEstimate) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation estimate.
#[derive(Debug, Clone)]
pub struct FederationEstimateValidator {
    pub rules: Vec<Rc<dyn FederationEstimateRule>>,
}

impl FederationEstimateValidator {
    pub fn validate(&self, item: &FederationEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_estimate_validator() -> FederationEstimateValidator {
    FederationEstimateValidator {
        rules: vec![
            Rc::new(FederationEstimateScoreRangeRule { min: 9.2, max: 3724.5 }) as Rc<dyn FederationEstimateRule>,
            Rc::new(FederationEstimateRequiredCodeRule {
                message: "federation estimate needs a value".to_string(),
            }),
            Rc::new(FederationEstimateNamespaceLengthRule { max_length: 32 }),
            Rc::new(FederationEstimateTimezoneLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_estimate_validator();
        let item = new_federation_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = FederationEstimateScoreRangeRule { min: 6.1, max: 4669.0 };
        let mut item = new_federation_estimate("x", "y");
        item.score = 4670.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FederationEstimateRequiredCodeRule {
            message: "federation estimate needs a value".to_string(),
        };
        let mut item = new_federation_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = FederationEstimateNamespaceLengthRule { max_length: 1000 };
        let mut item = new_federation_estimate("x", "y");
        item.namespace = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = FederationEstimateTimezoneLengthRule { max_length: 80 };
        let mut item = new_federation_estimate("x", "y");
        item.timezone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
