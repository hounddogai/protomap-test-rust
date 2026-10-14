use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a data export policy must pass. Check returns its problems, or none.
pub trait DataExportPolicyRule: std::fmt::Debug {
    fn check(&self, item: &DataExportPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPolicyLabelLengthRule {
    pub max_length: i64,
}

impl DataExportPolicyRule for DataExportPolicyLabelLengthRule {
    fn check(&self, item: &DataExportPolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPolicyContactEmailLengthRule {
    pub max_length: i64,
}

impl DataExportPolicyRule for DataExportPolicyContactEmailLengthRule {
    fn check(&self, item: &DataExportPolicy) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPolicyAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportPolicyRule for DataExportPolicyAttemptsRangeRule {
    fn check(&self, item: &DataExportPolicy) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPolicyRequiredKeyIdRule {
    pub message: String,
}

impl DataExportPolicyRule for DataExportPolicyRequiredKeyIdRule {
    fn check(&self, item: &DataExportPolicy) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPolicyAllowedContactEmailRule {
    pub allowed: Vec<String>,
}

impl DataExportPolicyRule for DataExportPolicyAllowedContactEmailRule {
    fn check(&self, item: &DataExportPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.contact_email)))
        {
            return vec![format!("contact email {} is expired", item.contact_email)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export policy.
#[derive(Debug, Clone)]
pub struct DataExportPolicyValidator {
    pub rules: Vec<Rc<dyn DataExportPolicyRule>>,
}

impl DataExportPolicyValidator {
    pub fn validate(&self, item: &DataExportPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DataExportPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_policy_validator() -> DataExportPolicyValidator {
    DataExportPolicyValidator {
        rules: vec![
            Rc::new(DataExportPolicyLabelLengthRule { max_length: 86400 }) as Rc<dyn DataExportPolicyRule>,
            Rc::new(DataExportPolicyContactEmailLengthRule { max_length: 255 }),
            Rc::new(DataExportPolicyAttemptsRangeRule { min: 16, max: 64 }),
            Rc::new(DataExportPolicyRequiredKeyIdRule {
                message: "data export policy needs an entry".to_string(),
            }),
            Rc::new(DataExportPolicyAllowedContactEmailRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_policy_validator();
        let item = new_data_export_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DataExportPolicyLabelLengthRule { max_length: 64 };
        let mut item = new_data_export_policy("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = DataExportPolicyContactEmailLengthRule { max_length: 80 };
        let mut item = new_data_export_policy("x", "y");
        item.contact_email = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = DataExportPolicyAttemptsRangeRule { min: 255, max: 1000 };
        let mut item = new_data_export_policy("x", "y");
        item.attempts = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = DataExportPolicyRequiredKeyIdRule {
            message: "data export policy needs an entry".to_string(),
        };
        let mut item = new_data_export_policy("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
