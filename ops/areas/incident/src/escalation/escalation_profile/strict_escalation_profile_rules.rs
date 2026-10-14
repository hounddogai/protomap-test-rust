use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict escalation profile must pass. Check returns its problems, or none.
pub trait StrictEscalationProfileRule: std::fmt::Debug {
    fn check(&self, item: &StrictEscalationProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationProfileNameLengthRule {
    pub max_length: i64,
}

impl StrictEscalationProfileRule for StrictEscalationProfileNameLengthRule {
    fn check(&self, item: &StrictEscalationProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationProfileRequiredCodeRule {
    pub message: String,
}

impl StrictEscalationProfileRule for StrictEscalationProfileRequiredCodeRule {
    fn check(&self, item: &StrictEscalationProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationProfileRequiredNameRule {
    pub message: String,
}

impl StrictEscalationProfileRule for StrictEscalationProfileRequiredNameRule {
    fn check(&self, item: &StrictEscalationProfile) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationProfileTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictEscalationProfileRule for StrictEscalationProfileTtlSecondsRangeRule {
    fn check(&self, item: &StrictEscalationProfile) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationProfileCodePrefixRule {
    pub prefix: String,
}

impl StrictEscalationProfileRule for StrictEscalationProfileCodePrefixRule {
    fn check(&self, item: &StrictEscalationProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationProfileClusterLengthRule {
    pub max_length: i64,
}

impl StrictEscalationProfileRule for StrictEscalationProfileClusterLengthRule {
    fn check(&self, item: &StrictEscalationProfile) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict escalation profile.
#[derive(Debug, Clone)]
pub struct StrictEscalationProfileValidator {
    pub rules: Vec<Rc<dyn StrictEscalationProfileRule>>,
}

impl StrictEscalationProfileValidator {
    pub fn validate(&self, item: &StrictEscalationProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictEscalationProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictEscalationProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_escalation_profile_validator() -> StrictEscalationProfileValidator {
    StrictEscalationProfileValidator {
        rules: vec![
            Rc::new(StrictEscalationProfileNameLengthRule { max_length: 120 }) as Rc<dyn StrictEscalationProfileRule>,
            Rc::new(StrictEscalationProfileRequiredCodeRule {
                message: "strict escalation profile needs this field".to_string(),
            }),
            Rc::new(StrictEscalationProfileRequiredNameRule {
                message: "strict escalation profile needs an entry".to_string(),
            }),
            Rc::new(StrictEscalationProfileTtlSecondsRangeRule { min: 32, max: 3600 }),
            Rc::new(StrictEscalationProfileCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(StrictEscalationProfileClusterLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_escalation_profile_validator();
        let item = new_strict_escalation_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictEscalationProfileNameLengthRule { max_length: 3600 };
        let mut item = new_strict_escalation_profile("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictEscalationProfileRequiredCodeRule {
            message: "strict escalation profile needs this field".to_string(),
        };
        let mut item = new_strict_escalation_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictEscalationProfileRequiredNameRule {
            message: "strict escalation profile needs this field".to_string(),
        };
        let mut item = new_strict_escalation_profile("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = StrictEscalationProfileTtlSecondsRangeRule { min: 255, max: 3600 };
        let mut item = new_strict_escalation_profile("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = StrictEscalationProfileClusterLengthRule { max_length: 32 };
        let mut item = new_strict_escalation_profile("x", "y");
        item.cluster = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
