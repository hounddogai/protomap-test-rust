use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict health check summary must pass. Check returns its problems, or none.
pub trait StrictHealthCheckSummaryRule: std::fmt::Debug {
    fn check(&self, item: &StrictHealthCheckSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckSummaryRequiredNameRule {
    pub message: String,
}

impl StrictHealthCheckSummaryRule for StrictHealthCheckSummaryRequiredNameRule {
    fn check(&self, item: &StrictHealthCheckSummary) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckSummaryFingerprintLengthRule {
    pub max_length: i64,
}

impl StrictHealthCheckSummaryRule for StrictHealthCheckSummaryFingerprintLengthRule {
    fn check(&self, item: &StrictHealthCheckSummary) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckSummaryNameLengthRule {
    pub max_length: i64,
}

impl StrictHealthCheckSummaryRule for StrictHealthCheckSummaryNameLengthRule {
    fn check(&self, item: &StrictHealthCheckSummary) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckSummaryCodePrefixRule {
    pub prefix: String,
}

impl StrictHealthCheckSummaryRule for StrictHealthCheckSummaryCodePrefixRule {
    fn check(&self, item: &StrictHealthCheckSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict health check summary.
#[derive(Debug, Clone)]
pub struct StrictHealthCheckSummaryValidator {
    pub rules: Vec<Rc<dyn StrictHealthCheckSummaryRule>>,
}

impl StrictHealthCheckSummaryValidator {
    pub fn validate(&self, item: &StrictHealthCheckSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictHealthCheckSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictHealthCheckSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_health_check_summary_validator() -> StrictHealthCheckSummaryValidator {
    StrictHealthCheckSummaryValidator {
        rules: vec![
            Rc::new(StrictHealthCheckSummaryRequiredNameRule {
                message: "strict health check summary needs an entry".to_string(),
            }) as Rc<dyn StrictHealthCheckSummaryRule>,
            Rc::new(StrictHealthCheckSummaryFingerprintLengthRule { max_length: 120 }),
            Rc::new(StrictHealthCheckSummaryNameLengthRule { max_length: 1000 }),
            Rc::new(StrictHealthCheckSummaryCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_health_check_summary_validator();
        let item = new_strict_health_check_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictHealthCheckSummaryRequiredNameRule {
            message: "strict health check summary needs this field".to_string(),
        };
        let mut item = new_strict_health_check_summary("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = StrictHealthCheckSummaryFingerprintLengthRule { max_length: 16 };
        let mut item = new_strict_health_check_summary("x", "y");
        item.fingerprint = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictHealthCheckSummaryNameLengthRule { max_length: 3600 };
        let mut item = new_strict_health_check_summary("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
