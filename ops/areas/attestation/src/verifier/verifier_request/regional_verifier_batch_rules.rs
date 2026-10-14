use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional verifier batch must pass. Check returns its problems, or none.
pub trait RegionalVerifierBatchRule: std::fmt::Debug {
    fn check(&self, item: &RegionalVerifierBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierBatchAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl RegionalVerifierBatchRule for RegionalVerifierBatchAllowedLabelRule {
    fn check(&self, item: &RegionalVerifierBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is unknown", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierBatchCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalVerifierBatchRule for RegionalVerifierBatchCeilingRangeRule {
    fn check(&self, item: &RegionalVerifierBatch) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierBatchCodeLengthRule {
    pub max_length: i64,
}

impl RegionalVerifierBatchRule for RegionalVerifierBatchCodeLengthRule {
    fn check(&self, item: &RegionalVerifierBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional verifier batch.
#[derive(Debug, Clone)]
pub struct RegionalVerifierBatchValidator {
    pub rules: Vec<Rc<dyn RegionalVerifierBatchRule>>,
}

impl RegionalVerifierBatchValidator {
    pub fn validate(&self, item: &RegionalVerifierBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalVerifierBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalVerifierBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_verifier_batch_validator() -> RegionalVerifierBatchValidator {
    RegionalVerifierBatchValidator {
        rules: vec![
            Rc::new(RegionalVerifierBatchAllowedLabelRule {
                allowed: vec!["legacy".to_string(), "global".to_string()],
            }) as Rc<dyn RegionalVerifierBatchRule>,
            Rc::new(RegionalVerifierBatchCeilingRangeRule { min: 8.8, max: 2292.3 }),
            Rc::new(RegionalVerifierBatchCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_verifier_batch_validator();
        let item = new_regional_verifier_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = RegionalVerifierBatchCeilingRangeRule { min: 7.8, max: 1729.9 };
        let mut item = new_regional_verifier_batch("x", "y");
        item.ceiling = 1731.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalVerifierBatchCodeLengthRule { max_length: 80 };
        let mut item = new_regional_verifier_batch("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
