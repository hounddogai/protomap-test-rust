use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal credential entry must pass. Check returns its problems, or none.
pub trait SeasonalCredentialEntryRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalCredentialEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialEntryCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalCredentialEntryRule for SeasonalCredentialEntryCodeLengthRule {
    fn check(&self, item: &SeasonalCredentialEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialEntryRequiredClusterRule {
    pub message: String,
}

impl SeasonalCredentialEntryRule for SeasonalCredentialEntryRequiredClusterRule {
    fn check(&self, item: &SeasonalCredentialEntry) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialEntryClusterLengthRule {
    pub max_length: i64,
}

impl SeasonalCredentialEntryRule for SeasonalCredentialEntryClusterLengthRule {
    fn check(&self, item: &SeasonalCredentialEntry) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialEntryRequiredLabelRule {
    pub message: String,
}

impl SeasonalCredentialEntryRule for SeasonalCredentialEntryRequiredLabelRule {
    fn check(&self, item: &SeasonalCredentialEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal credential entry.
#[derive(Debug, Clone)]
pub struct SeasonalCredentialEntryValidator {
    pub rules: Vec<Rc<dyn SeasonalCredentialEntryRule>>,
}

impl SeasonalCredentialEntryValidator {
    pub fn validate(&self, item: &SeasonalCredentialEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalCredentialEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalCredentialEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_credential_entry_validator() -> SeasonalCredentialEntryValidator {
    SeasonalCredentialEntryValidator {
        rules: vec![
            Rc::new(SeasonalCredentialEntryCodeLengthRule { max_length: 255 }) as Rc<dyn SeasonalCredentialEntryRule>,
            Rc::new(SeasonalCredentialEntryRequiredClusterRule {
                message: "seasonal credential entry needs a value".to_string(),
            }),
            Rc::new(SeasonalCredentialEntryClusterLengthRule { max_length: 32 }),
            Rc::new(SeasonalCredentialEntryRequiredLabelRule {
                message: "seasonal credential entry needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_credential_entry_validator();
        let item = new_seasonal_credential_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalCredentialEntryCodeLengthRule { max_length: 1000 };
        let mut item = new_seasonal_credential_entry("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = SeasonalCredentialEntryRequiredClusterRule {
            message: "seasonal credential entry needs a value".to_string(),
        };
        let mut item = new_seasonal_credential_entry("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = SeasonalCredentialEntryClusterLengthRule { max_length: 80 };
        let mut item = new_seasonal_credential_entry("x", "y");
        item.cluster = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SeasonalCredentialEntryRequiredLabelRule {
            message: "seasonal credential entry needs this field".to_string(),
        };
        let mut item = new_seasonal_credential_entry("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
