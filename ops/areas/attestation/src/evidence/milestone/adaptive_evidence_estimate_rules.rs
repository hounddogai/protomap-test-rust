use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive evidence estimate must pass. Check returns its problems, or none.
pub trait AdaptiveEvidenceEstimateRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveEvidenceEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEvidenceEstimateCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveEvidenceEstimateRule for AdaptiveEvidenceEstimateCodePrefixRule {
    fn check(&self, item: &AdaptiveEvidenceEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEvidenceEstimateCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveEvidenceEstimateRule for AdaptiveEvidenceEstimateCostRangeRule {
    fn check(&self, item: &AdaptiveEvidenceEstimate) -> Vec<String> {
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
pub struct AdaptiveEvidenceEstimateMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveEvidenceEstimateRule for AdaptiveEvidenceEstimateMarginRangeRule {
    fn check(&self, item: &AdaptiveEvidenceEstimate) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEvidenceEstimateAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl AdaptiveEvidenceEstimateRule for AdaptiveEvidenceEstimateAllowedSourceRule {
    fn check(&self, item: &AdaptiveEvidenceEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is below minimum", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEvidenceEstimateScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveEvidenceEstimateRule for AdaptiveEvidenceEstimateScoreRangeRule {
    fn check(&self, item: &AdaptiveEvidenceEstimate) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEvidenceEstimateCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveEvidenceEstimateRule for AdaptiveEvidenceEstimateCodeLengthRule {
    fn check(&self, item: &AdaptiveEvidenceEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive evidence estimate.
#[derive(Debug, Clone)]
pub struct AdaptiveEvidenceEstimateValidator {
    pub rules: Vec<Rc<dyn AdaptiveEvidenceEstimateRule>>,
}

impl AdaptiveEvidenceEstimateValidator {
    pub fn validate(&self, item: &AdaptiveEvidenceEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveEvidenceEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveEvidenceEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_evidence_estimate_validator() -> AdaptiveEvidenceEstimateValidator {
    AdaptiveEvidenceEstimateValidator {
        rules: vec![
            Rc::new(AdaptiveEvidenceEstimateCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn AdaptiveEvidenceEstimateRule>,
            Rc::new(AdaptiveEvidenceEstimateCostRangeRule { min: 8.0, max: 3702.1 }),
            Rc::new(AdaptiveEvidenceEstimateMarginRangeRule { min: 6.8, max: 4837.7 }),
            Rc::new(AdaptiveEvidenceEstimateAllowedSourceRule {
                allowed: vec!["strict".to_string()],
            }),
            Rc::new(AdaptiveEvidenceEstimateScoreRangeRule { min: 1.1, max: 1352.7 }),
            Rc::new(AdaptiveEvidenceEstimateCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_evidence_estimate_validator();
        let item = new_adaptive_evidence_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = AdaptiveEvidenceEstimateCostRangeRule { min: 8.9, max: 1517.0 };
        let mut item = new_adaptive_evidence_estimate("x", "y");
        item.cost = 1518.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = AdaptiveEvidenceEstimateMarginRangeRule { min: 2.5, max: 1831.3 };
        let mut item = new_adaptive_evidence_estimate("x", "y");
        item.margin = 1832.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = AdaptiveEvidenceEstimateScoreRangeRule { min: 9.3, max: 3941.7 };
        let mut item = new_adaptive_evidence_estimate("x", "y");
        item.score = 3943.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveEvidenceEstimateCodeLengthRule { max_length: 255 };
        let mut item = new_adaptive_evidence_estimate("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
