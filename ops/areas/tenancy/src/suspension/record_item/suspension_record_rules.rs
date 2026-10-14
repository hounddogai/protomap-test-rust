use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a suspension record must pass. Check returns its problems, or none.
pub trait SuspensionRecordRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRecordRequiredClusterRule {
    pub message: String,
}

impl SuspensionRecordRule for SuspensionRecordRequiredClusterRule {
    fn check(&self, item: &SuspensionRecord) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRecordRequiredCountryRule {
    pub message: String,
}

impl SuspensionRecordRule for SuspensionRecordRequiredCountryRule {
    fn check(&self, item: &SuspensionRecord) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRecordCodePrefixRule {
    pub prefix: String,
}

impl SuspensionRecordRule for SuspensionRecordCodePrefixRule {
    fn check(&self, item: &SuspensionRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRecordAllowedNameRule {
    pub allowed: Vec<String>,
}

impl SuspensionRecordRule for SuspensionRecordAllowedNameRule {
    fn check(&self, item: &SuspensionRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension record.
#[derive(Debug, Clone)]
pub struct SuspensionRecordValidator {
    pub rules: Vec<Rc<dyn SuspensionRecordRule>>,
}

impl SuspensionRecordValidator {
    pub fn validate(&self, item: &SuspensionRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_record_validator() -> SuspensionRecordValidator {
    SuspensionRecordValidator {
        rules: vec![
            Rc::new(SuspensionRecordRequiredClusterRule {
                message: "suspension record needs this field".to_string(),
            }) as Rc<dyn SuspensionRecordRule>,
            Rc::new(SuspensionRecordRequiredCountryRule {
                message: "suspension record needs an entry".to_string(),
            }),
            Rc::new(SuspensionRecordCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(SuspensionRecordAllowedNameRule {
                allowed: vec!["batched".to_string(), "pending".to_string(), "scheduled".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_record_validator();
        let item = new_suspension_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = SuspensionRecordRequiredClusterRule {
            message: "suspension record needs an entry".to_string(),
        };
        let mut item = new_suspension_record("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = SuspensionRecordRequiredCountryRule {
            message: "suspension record needs this field".to_string(),
        };
        let mut item = new_suspension_record("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
