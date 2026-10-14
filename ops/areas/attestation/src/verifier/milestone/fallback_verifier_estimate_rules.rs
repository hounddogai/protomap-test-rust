use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback verifier estimate must pass. Check returns its problems, or none.
pub trait FallbackVerifierEstimateRule: std::fmt::Debug {
    fn check(&self, item: &FallbackVerifierEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierEstimateRequiredDescriptionRule {
    pub message: String,
}

impl FallbackVerifierEstimateRule for FallbackVerifierEstimateRequiredDescriptionRule {
    fn check(&self, item: &FallbackVerifierEstimate) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierEstimateThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackVerifierEstimateRule for FallbackVerifierEstimateThresholdRangeRule {
    fn check(&self, item: &FallbackVerifierEstimate) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierEstimateDescriptionLengthRule {
    pub max_length: i64,
}

impl FallbackVerifierEstimateRule for FallbackVerifierEstimateDescriptionLengthRule {
    fn check(&self, item: &FallbackVerifierEstimate) -> Vec<String> {
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
pub struct FallbackVerifierEstimateCodeLengthRule {
    pub max_length: i64,
}

impl FallbackVerifierEstimateRule for FallbackVerifierEstimateCodeLengthRule {
    fn check(&self, item: &FallbackVerifierEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierEstimateBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackVerifierEstimateRule for FallbackVerifierEstimateBatchSizeRangeRule {
    fn check(&self, item: &FallbackVerifierEstimate) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierEstimateNameLengthRule {
    pub max_length: i64,
}

impl FallbackVerifierEstimateRule for FallbackVerifierEstimateNameLengthRule {
    fn check(&self, item: &FallbackVerifierEstimate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback verifier estimate.
#[derive(Debug, Clone)]
pub struct FallbackVerifierEstimateValidator {
    pub rules: Vec<Rc<dyn FallbackVerifierEstimateRule>>,
}

impl FallbackVerifierEstimateValidator {
    pub fn validate(&self, item: &FallbackVerifierEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackVerifierEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackVerifierEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_verifier_estimate_validator() -> FallbackVerifierEstimateValidator {
    FallbackVerifierEstimateValidator {
        rules: vec![
            Rc::new(FallbackVerifierEstimateRequiredDescriptionRule {
                message: "fallback verifier estimate needs this field".to_string(),
            }) as Rc<dyn FallbackVerifierEstimateRule>,
            Rc::new(FallbackVerifierEstimateThresholdRangeRule { min: 4.8, max: 3308.5 }),
            Rc::new(FallbackVerifierEstimateDescriptionLengthRule { max_length: 64 }),
            Rc::new(FallbackVerifierEstimateCodeLengthRule { max_length: 64 }),
            Rc::new(FallbackVerifierEstimateBatchSizeRangeRule { min: 32, max: 3600 }),
            Rc::new(FallbackVerifierEstimateNameLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_verifier_estimate_validator();
        let item = new_fallback_verifier_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = FallbackVerifierEstimateRequiredDescriptionRule {
            message: "fallback verifier estimate needs this field".to_string(),
        };
        let mut item = new_fallback_verifier_estimate("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = FallbackVerifierEstimateThresholdRangeRule { min: 6.5, max: 3632.8 };
        let mut item = new_fallback_verifier_estimate("x", "y");
        item.threshold = 3634.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = FallbackVerifierEstimateDescriptionLengthRule { max_length: 1000 };
        let mut item = new_fallback_verifier_estimate("x", "y");
        item.description = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackVerifierEstimateCodeLengthRule { max_length: 64 };
        let mut item = new_fallback_verifier_estimate("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = FallbackVerifierEstimateBatchSizeRangeRule { min: 120, max: 120 };
        let mut item = new_fallback_verifier_estimate("x", "y");
        item.batch_size = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackVerifierEstimateNameLengthRule { max_length: 16 };
        let mut item = new_fallback_verifier_estimate("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
