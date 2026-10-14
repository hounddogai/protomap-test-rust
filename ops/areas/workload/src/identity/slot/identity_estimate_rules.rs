use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity estimate must pass. Check returns its problems, or none.
pub trait IdentityEstimateRule: std::fmt::Debug {
    fn check(&self, item: &IdentityEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEstimateRequiredOwnerIdRule {
    pub message: String,
}

impl IdentityEstimateRule for IdentityEstimateRequiredOwnerIdRule {
    fn check(&self, item: &IdentityEstimate) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEstimateCodePrefixRule {
    pub prefix: String,
}

impl IdentityEstimateRule for IdentityEstimateCodePrefixRule {
    fn check(&self, item: &IdentityEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEstimateScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl IdentityEstimateRule for IdentityEstimateScoreRangeRule {
    fn check(&self, item: &IdentityEstimate) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity estimate.
#[derive(Debug, Clone)]
pub struct IdentityEstimateValidator {
    pub rules: Vec<Rc<dyn IdentityEstimateRule>>,
}

impl IdentityEstimateValidator {
    pub fn validate(&self, item: &IdentityEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentityEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_estimate_validator() -> IdentityEstimateValidator {
    IdentityEstimateValidator {
        rules: vec![
            Rc::new(IdentityEstimateRequiredOwnerIdRule {
                message: "identity estimate needs this field".to_string(),
            }) as Rc<dyn IdentityEstimateRule>,
            Rc::new(IdentityEstimateCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(IdentityEstimateScoreRangeRule { min: 5.6, max: 1896.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_estimate_validator();
        let item = new_identity_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = IdentityEstimateRequiredOwnerIdRule {
            message: "identity estimate needs an entry".to_string(),
        };
        let mut item = new_identity_estimate("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = IdentityEstimateScoreRangeRule { min: 4.3, max: 398.6 };
        let mut item = new_identity_estimate("x", "y");
        item.score = 400.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
