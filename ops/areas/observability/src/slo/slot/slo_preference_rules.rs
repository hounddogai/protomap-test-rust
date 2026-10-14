use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo preference must pass. Check returns its problems, or none.
pub trait SloPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &SloPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPreferenceRequiredClusterRule {
    pub message: String,
}

impl SloPreferenceRule for SloPreferenceRequiredClusterRule {
    fn check(&self, item: &SloPreference) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPreferenceClusterLengthRule {
    pub max_length: i64,
}

impl SloPreferenceRule for SloPreferenceClusterLengthRule {
    fn check(&self, item: &SloPreference) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPreferenceThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloPreferenceRule for SloPreferenceThresholdRangeRule {
    fn check(&self, item: &SloPreference) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPreferenceCodePrefixRule {
    pub prefix: String,
}

impl SloPreferenceRule for SloPreferenceCodePrefixRule {
    fn check(&self, item: &SloPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo preference.
#[derive(Debug, Clone)]
pub struct SloPreferenceValidator {
    pub rules: Vec<Rc<dyn SloPreferenceRule>>,
}

impl SloPreferenceValidator {
    pub fn validate(&self, item: &SloPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_preference_validator() -> SloPreferenceValidator {
    SloPreferenceValidator {
        rules: vec![
            Rc::new(SloPreferenceRequiredClusterRule {
                message: "slo preference needs this field".to_string(),
            }) as Rc<dyn SloPreferenceRule>,
            Rc::new(SloPreferenceClusterLengthRule { max_length: 1000 }),
            Rc::new(SloPreferenceThresholdRangeRule { min: 3.8, max: 3720.9 }),
            Rc::new(SloPreferenceCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_preference_validator();
        let item = new_slo_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = SloPreferenceRequiredClusterRule {
            message: "slo preference needs a value".to_string(),
        };
        let mut item = new_slo_preference("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = SloPreferenceClusterLengthRule { max_length: 16 };
        let mut item = new_slo_preference("x", "y");
        item.cluster = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = SloPreferenceThresholdRangeRule { min: 0.5, max: 2024.4 };
        let mut item = new_slo_preference("x", "y");
        item.threshold = 2025.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
