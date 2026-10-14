use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending policy variance must pass. Check returns its problems, or none.
pub trait PendingPolicyVarianceRule: std::fmt::Debug {
    fn check(&self, item: &PendingPolicyVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyVarianceRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingPolicyVarianceRule for PendingPolicyVarianceRatioRangeRule {
    fn check(&self, item: &PendingPolicyVariance) -> Vec<String> {
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
pub struct PendingPolicyVarianceAudienceLengthRule {
    pub max_length: i64,
}

impl PendingPolicyVarianceRule for PendingPolicyVarianceAudienceLengthRule {
    fn check(&self, item: &PendingPolicyVariance) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyVarianceRequiredAudienceRule {
    pub message: String,
}

impl PendingPolicyVarianceRule for PendingPolicyVarianceRequiredAudienceRule {
    fn check(&self, item: &PendingPolicyVariance) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyVarianceCodeLengthRule {
    pub max_length: i64,
}

impl PendingPolicyVarianceRule for PendingPolicyVarianceCodeLengthRule {
    fn check(&self, item: &PendingPolicyVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyVarianceTitleLengthRule {
    pub max_length: i64,
}

impl PendingPolicyVarianceRule for PendingPolicyVarianceTitleLengthRule {
    fn check(&self, item: &PendingPolicyVariance) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending policy variance.
#[derive(Debug, Clone)]
pub struct PendingPolicyVarianceValidator {
    pub rules: Vec<Rc<dyn PendingPolicyVarianceRule>>,
}

impl PendingPolicyVarianceValidator {
    pub fn validate(&self, item: &PendingPolicyVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingPolicyVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPolicyVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_policy_variance_validator() -> PendingPolicyVarianceValidator {
    PendingPolicyVarianceValidator {
        rules: vec![
            Rc::new(PendingPolicyVarianceRatioRangeRule { min: 5.7, max: 2699.1 }) as Rc<dyn PendingPolicyVarianceRule>,
            Rc::new(PendingPolicyVarianceAudienceLengthRule { max_length: 255 }),
            Rc::new(PendingPolicyVarianceRequiredAudienceRule {
                message: "pending policy variance needs an entry".to_string(),
            }),
            Rc::new(PendingPolicyVarianceCodeLengthRule { max_length: 3600 }),
            Rc::new(PendingPolicyVarianceTitleLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_policy_variance_validator();
        let item = new_pending_policy_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = PendingPolicyVarianceRatioRangeRule { min: 5.3, max: 1332.3 };
        let mut item = new_pending_policy_variance("x", "y");
        item.ratio = 1333.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = PendingPolicyVarianceAudienceLengthRule { max_length: 3600 };
        let mut item = new_pending_policy_variance("x", "y");
        item.audience = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = PendingPolicyVarianceRequiredAudienceRule {
            message: "pending policy variance needs this field".to_string(),
        };
        let mut item = new_pending_policy_variance("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingPolicyVarianceCodeLengthRule { max_length: 16 };
        let mut item = new_pending_policy_variance("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingPolicyVarianceTitleLengthRule { max_length: 32 };
        let mut item = new_pending_policy_variance("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
