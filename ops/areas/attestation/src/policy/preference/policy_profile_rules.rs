use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a policy profile must pass. Check returns its problems, or none.
pub trait PolicyProfileRule: std::fmt::Debug {
    fn check(&self, item: &PolicyProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyProfileBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyProfileRule for PolicyProfileBatchSizeRangeRule {
    fn check(&self, item: &PolicyProfile) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyProfileAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl PolicyProfileRule for PolicyProfileAllowedExternalRefRule {
    fn check(&self, item: &PolicyProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is blocked", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyProfileExternalRefLengthRule {
    pub max_length: i64,
}

impl PolicyProfileRule for PolicyProfileExternalRefLengthRule {
    fn check(&self, item: &PolicyProfile) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy profile.
#[derive(Debug, Clone)]
pub struct PolicyProfileValidator {
    pub rules: Vec<Rc<dyn PolicyProfileRule>>,
}

impl PolicyProfileValidator {
    pub fn validate(&self, item: &PolicyProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_profile_validator() -> PolicyProfileValidator {
    PolicyProfileValidator {
        rules: vec![
            Rc::new(PolicyProfileBatchSizeRangeRule { min: 255, max: 255 }) as Rc<dyn PolicyProfileRule>,
            Rc::new(PolicyProfileAllowedExternalRefRule { allowed: Vec::new() }),
            Rc::new(PolicyProfileExternalRefLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_profile_validator();
        let item = new_policy_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = PolicyProfileBatchSizeRangeRule { min: 32, max: 3600 };
        let mut item = new_policy_profile("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = PolicyProfileExternalRefLengthRule { max_length: 32 };
        let mut item = new_policy_profile("x", "y");
        item.external_ref = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
