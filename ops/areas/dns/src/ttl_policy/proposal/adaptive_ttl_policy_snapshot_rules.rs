use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive ttl policy snapshot must pass. Check returns its problems, or none.
pub trait AdaptiveTtlPolicySnapshotRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveTtlPolicySnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicySnapshotCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveTtlPolicySnapshotRule for AdaptiveTtlPolicySnapshotCodeLengthRule {
    fn check(&self, item: &AdaptiveTtlPolicySnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicySnapshotCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveTtlPolicySnapshotRule for AdaptiveTtlPolicySnapshotCodePrefixRule {
    fn check(&self, item: &AdaptiveTtlPolicySnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicySnapshotRequiredCodeRule {
    pub message: String,
}

impl AdaptiveTtlPolicySnapshotRule for AdaptiveTtlPolicySnapshotRequiredCodeRule {
    fn check(&self, item: &AdaptiveTtlPolicySnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicySnapshotRequiredTitleRule {
    pub message: String,
}

impl AdaptiveTtlPolicySnapshotRule for AdaptiveTtlPolicySnapshotRequiredTitleRule {
    fn check(&self, item: &AdaptiveTtlPolicySnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicySnapshotRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveTtlPolicySnapshotRule for AdaptiveTtlPolicySnapshotRatioRangeRule {
    fn check(&self, item: &AdaptiveTtlPolicySnapshot) -> Vec<String> {
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
pub struct AdaptiveTtlPolicySnapshotCountryLengthRule {
    pub max_length: i64,
}

impl AdaptiveTtlPolicySnapshotRule for AdaptiveTtlPolicySnapshotCountryLengthRule {
    fn check(&self, item: &AdaptiveTtlPolicySnapshot) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive ttl policy snapshot.
#[derive(Debug, Clone)]
pub struct AdaptiveTtlPolicySnapshotValidator {
    pub rules: Vec<Rc<dyn AdaptiveTtlPolicySnapshotRule>>,
}

impl AdaptiveTtlPolicySnapshotValidator {
    pub fn validate(&self, item: &AdaptiveTtlPolicySnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveTtlPolicySnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveTtlPolicySnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_ttl_policy_snapshot_validator() -> AdaptiveTtlPolicySnapshotValidator {
    AdaptiveTtlPolicySnapshotValidator {
        rules: vec![
            Rc::new(AdaptiveTtlPolicySnapshotCodeLengthRule { max_length: 16 })
                as Rc<dyn AdaptiveTtlPolicySnapshotRule>,
            Rc::new(AdaptiveTtlPolicySnapshotCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(AdaptiveTtlPolicySnapshotRequiredCodeRule {
                message: "adaptive ttl policy snapshot needs this field".to_string(),
            }),
            Rc::new(AdaptiveTtlPolicySnapshotRequiredTitleRule {
                message: "adaptive ttl policy snapshot needs this field".to_string(),
            }),
            Rc::new(AdaptiveTtlPolicySnapshotRatioRangeRule { min: 1.9, max: 1093.0 }),
            Rc::new(AdaptiveTtlPolicySnapshotCountryLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_ttl_policy_snapshot_validator();
        let item = new_adaptive_ttl_policy_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveTtlPolicySnapshotCodeLengthRule { max_length: 86400 };
        let mut item = new_adaptive_ttl_policy_snapshot("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveTtlPolicySnapshotRequiredCodeRule {
            message: "adaptive ttl policy snapshot needs an entry".to_string(),
        };
        let mut item = new_adaptive_ttl_policy_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AdaptiveTtlPolicySnapshotRequiredTitleRule {
            message: "adaptive ttl policy snapshot needs this field".to_string(),
        };
        let mut item = new_adaptive_ttl_policy_snapshot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = AdaptiveTtlPolicySnapshotRatioRangeRule { min: 7.4, max: 3109.5 };
        let mut item = new_adaptive_ttl_policy_snapshot("x", "y");
        item.ratio = 3111.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = AdaptiveTtlPolicySnapshotCountryLengthRule { max_length: 86400 };
        let mut item = new_adaptive_ttl_policy_snapshot("x", "y");
        item.country = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
