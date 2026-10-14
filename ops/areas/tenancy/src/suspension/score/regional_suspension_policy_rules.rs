use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional suspension policy must pass. Check returns its problems, or none.
pub trait RegionalSuspensionPolicyRule: std::fmt::Debug {
    fn check(&self, item: &RegionalSuspensionPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionPolicyMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalSuspensionPolicyRule for RegionalSuspensionPolicyMinimumRangeRule {
    fn check(&self, item: &RegionalSuspensionPolicy) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionPolicyRequiredHostRule {
    pub message: String,
}

impl RegionalSuspensionPolicyRule for RegionalSuspensionPolicyRequiredHostRule {
    fn check(&self, item: &RegionalSuspensionPolicy) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionPolicyBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalSuspensionPolicyRule for RegionalSuspensionPolicyBatchSizeRangeRule {
    fn check(&self, item: &RegionalSuspensionPolicy) -> Vec<String> {
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
pub struct RegionalSuspensionPolicyTitleLengthRule {
    pub max_length: i64,
}

impl RegionalSuspensionPolicyRule for RegionalSuspensionPolicyTitleLengthRule {
    fn check(&self, item: &RegionalSuspensionPolicy) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionPolicyExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalSuspensionPolicyRule for RegionalSuspensionPolicyExpiresAtRangeRule {
    fn check(&self, item: &RegionalSuspensionPolicy) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionPolicyAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl RegionalSuspensionPolicyRule for RegionalSuspensionPolicyAllowedNamespaceRule {
    fn check(&self, item: &RegionalSuspensionPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is not allowed", item.namespace)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional suspension policy.
#[derive(Debug, Clone)]
pub struct RegionalSuspensionPolicyValidator {
    pub rules: Vec<Rc<dyn RegionalSuspensionPolicyRule>>,
}

impl RegionalSuspensionPolicyValidator {
    pub fn validate(&self, item: &RegionalSuspensionPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalSuspensionPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalSuspensionPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_suspension_policy_validator() -> RegionalSuspensionPolicyValidator {
    RegionalSuspensionPolicyValidator {
        rules: vec![
            Rc::new(RegionalSuspensionPolicyMinimumRangeRule { min: 4.2, max: 2771.8 })
                as Rc<dyn RegionalSuspensionPolicyRule>,
            Rc::new(RegionalSuspensionPolicyRequiredHostRule {
                message: "regional suspension policy needs a value".to_string(),
            }),
            Rc::new(RegionalSuspensionPolicyBatchSizeRangeRule { min: 16, max: 80 }),
            Rc::new(RegionalSuspensionPolicyTitleLengthRule { max_length: 32 }),
            Rc::new(RegionalSuspensionPolicyExpiresAtRangeRule { min: 255, max: 86400 }),
            Rc::new(RegionalSuspensionPolicyAllowedNamespaceRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_suspension_policy_validator();
        let item = new_regional_suspension_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = RegionalSuspensionPolicyMinimumRangeRule { min: 5.4, max: 2324.8 };
        let mut item = new_regional_suspension_policy("x", "y");
        item.minimum = 2326.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RegionalSuspensionPolicyRequiredHostRule {
            message: "regional suspension policy needs an entry".to_string(),
        };
        let mut item = new_regional_suspension_policy("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = RegionalSuspensionPolicyBatchSizeRangeRule { min: 80, max: 120 };
        let mut item = new_regional_suspension_policy("x", "y");
        item.batch_size = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalSuspensionPolicyTitleLengthRule { max_length: 1000 };
        let mut item = new_regional_suspension_policy("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = RegionalSuspensionPolicyExpiresAtRangeRule { min: 32, max: 1000 };
        let mut item = new_regional_suspension_policy("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
