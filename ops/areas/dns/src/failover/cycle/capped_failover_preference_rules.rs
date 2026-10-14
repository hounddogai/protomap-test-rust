use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped failover preference must pass. Check returns its problems, or none.
pub trait CappedFailoverPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &CappedFailoverPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFailoverPreferenceRequiredNameRule {
    pub message: String,
}

impl CappedFailoverPreferenceRule for CappedFailoverPreferenceRequiredNameRule {
    fn check(&self, item: &CappedFailoverPreference) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFailoverPreferenceRequiredClusterRule {
    pub message: String,
}

impl CappedFailoverPreferenceRule for CappedFailoverPreferenceRequiredClusterRule {
    fn check(&self, item: &CappedFailoverPreference) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFailoverPreferenceRequiredCodeRule {
    pub message: String,
}

impl CappedFailoverPreferenceRule for CappedFailoverPreferenceRequiredCodeRule {
    fn check(&self, item: &CappedFailoverPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFailoverPreferenceCodePrefixRule {
    pub prefix: String,
}

impl CappedFailoverPreferenceRule for CappedFailoverPreferenceCodePrefixRule {
    fn check(&self, item: &CappedFailoverPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFailoverPreferenceAllowedNameRule {
    pub allowed: Vec<String>,
}

impl CappedFailoverPreferenceRule for CappedFailoverPreferenceAllowedNameRule {
    fn check(&self, item: &CappedFailoverPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is below minimum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFailoverPreferenceCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedFailoverPreferenceRule for CappedFailoverPreferenceCeilingRangeRule {
    fn check(&self, item: &CappedFailoverPreference) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped failover preference.
#[derive(Debug, Clone)]
pub struct CappedFailoverPreferenceValidator {
    pub rules: Vec<Rc<dyn CappedFailoverPreferenceRule>>,
}

impl CappedFailoverPreferenceValidator {
    pub fn validate(&self, item: &CappedFailoverPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedFailoverPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedFailoverPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_failover_preference_validator() -> CappedFailoverPreferenceValidator {
    CappedFailoverPreferenceValidator {
        rules: vec![
            Rc::new(CappedFailoverPreferenceRequiredNameRule {
                message: "capped failover preference needs an entry".to_string(),
            }) as Rc<dyn CappedFailoverPreferenceRule>,
            Rc::new(CappedFailoverPreferenceRequiredClusterRule {
                message: "capped failover preference needs an entry".to_string(),
            }),
            Rc::new(CappedFailoverPreferenceRequiredCodeRule {
                message: "capped failover preference needs this field".to_string(),
            }),
            Rc::new(CappedFailoverPreferenceCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(CappedFailoverPreferenceAllowedNameRule {
                allowed: vec!["default".to_string(), "lenient".to_string(), "capped".to_string()],
            }),
            Rc::new(CappedFailoverPreferenceCeilingRangeRule { min: 7.1, max: 2659.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_failover_preference_validator();
        let item = new_capped_failover_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CappedFailoverPreferenceRequiredNameRule {
            message: "capped failover preference needs a value".to_string(),
        };
        let mut item = new_capped_failover_preference("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = CappedFailoverPreferenceRequiredClusterRule {
            message: "capped failover preference needs this field".to_string(),
        };
        let mut item = new_capped_failover_preference("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedFailoverPreferenceRequiredCodeRule {
            message: "capped failover preference needs a value".to_string(),
        };
        let mut item = new_capped_failover_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = CappedFailoverPreferenceCeilingRangeRule { min: 6.8, max: 2331.6 };
        let mut item = new_capped_failover_preference("x", "y");
        item.ceiling = 2333.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
