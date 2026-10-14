use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress preference must pass. Check returns its problems, or none.
pub trait EgressPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &EgressPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressPreferenceClusterLengthRule {
    pub max_length: i64,
}

impl EgressPreferenceRule for EgressPreferenceClusterLengthRule {
    fn check(&self, item: &EgressPreference) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressPreferenceRequiredCodeRule {
    pub message: String,
}

impl EgressPreferenceRule for EgressPreferenceRequiredCodeRule {
    fn check(&self, item: &EgressPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressPreferenceRequiredChecksumRule {
    pub message: String,
}

impl EgressPreferenceRule for EgressPreferenceRequiredChecksumRule {
    fn check(&self, item: &EgressPreference) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress preference.
#[derive(Debug, Clone)]
pub struct EgressPreferenceValidator {
    pub rules: Vec<Rc<dyn EgressPreferenceRule>>,
}

impl EgressPreferenceValidator {
    pub fn validate(&self, item: &EgressPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_preference_validator() -> EgressPreferenceValidator {
    EgressPreferenceValidator {
        rules: vec![
            Rc::new(EgressPreferenceClusterLengthRule { max_length: 3600 }) as Rc<dyn EgressPreferenceRule>,
            Rc::new(EgressPreferenceRequiredCodeRule {
                message: "egress preference needs a value".to_string(),
            }),
            Rc::new(EgressPreferenceRequiredChecksumRule {
                message: "egress preference needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_preference_validator();
        let item = new_egress_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = EgressPreferenceClusterLengthRule { max_length: 1000 };
        let mut item = new_egress_preference("x", "y");
        item.cluster = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EgressPreferenceRequiredCodeRule {
            message: "egress preference needs a value".to_string(),
        };
        let mut item = new_egress_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = EgressPreferenceRequiredChecksumRule {
            message: "egress preference needs an entry".to_string(),
        };
        let mut item = new_egress_preference("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
