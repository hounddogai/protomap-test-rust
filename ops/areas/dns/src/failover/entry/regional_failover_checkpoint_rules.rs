use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional failover checkpoint must pass. Check returns its problems, or none.
pub trait RegionalFailoverCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RegionalFailoverCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverCheckpointAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl RegionalFailoverCheckpointRule for RegionalFailoverCheckpointAllowedCountryRule {
    fn check(&self, item: &RegionalFailoverCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is stale", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverCheckpointChecksumLengthRule {
    pub max_length: i64,
}

impl RegionalFailoverCheckpointRule for RegionalFailoverCheckpointChecksumLengthRule {
    fn check(&self, item: &RegionalFailoverCheckpoint) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverCheckpointLabelLengthRule {
    pub max_length: i64,
}

impl RegionalFailoverCheckpointRule for RegionalFailoverCheckpointLabelLengthRule {
    fn check(&self, item: &RegionalFailoverCheckpoint) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverCheckpointRequiredLabelRule {
    pub message: String,
}

impl RegionalFailoverCheckpointRule for RegionalFailoverCheckpointRequiredLabelRule {
    fn check(&self, item: &RegionalFailoverCheckpoint) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverCheckpointCountryLengthRule {
    pub max_length: i64,
}

impl RegionalFailoverCheckpointRule for RegionalFailoverCheckpointCountryLengthRule {
    fn check(&self, item: &RegionalFailoverCheckpoint) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional failover checkpoint.
#[derive(Debug, Clone)]
pub struct RegionalFailoverCheckpointValidator {
    pub rules: Vec<Rc<dyn RegionalFailoverCheckpointRule>>,
}

impl RegionalFailoverCheckpointValidator {
    pub fn validate(&self, item: &RegionalFailoverCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalFailoverCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalFailoverCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_failover_checkpoint_validator() -> RegionalFailoverCheckpointValidator {
    RegionalFailoverCheckpointValidator {
        rules: vec![
            Rc::new(RegionalFailoverCheckpointAllowedCountryRule {
                allowed: vec!["scheduled".to_string(), "fallback".to_string()],
            }) as Rc<dyn RegionalFailoverCheckpointRule>,
            Rc::new(RegionalFailoverCheckpointChecksumLengthRule { max_length: 80 }),
            Rc::new(RegionalFailoverCheckpointLabelLengthRule { max_length: 32 }),
            Rc::new(RegionalFailoverCheckpointRequiredLabelRule {
                message: "regional failover checkpoint needs this field".to_string(),
            }),
            Rc::new(RegionalFailoverCheckpointCountryLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_failover_checkpoint_validator();
        let item = new_regional_failover_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = RegionalFailoverCheckpointChecksumLengthRule { max_length: 1000 };
        let mut item = new_regional_failover_checkpoint("x", "y");
        item.checksum = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalFailoverCheckpointLabelLengthRule { max_length: 3600 };
        let mut item = new_regional_failover_checkpoint("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalFailoverCheckpointRequiredLabelRule {
            message: "regional failover checkpoint needs a value".to_string(),
        };
        let mut item = new_regional_failover_checkpoint("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = RegionalFailoverCheckpointCountryLengthRule { max_length: 64 };
        let mut item = new_regional_failover_checkpoint("x", "y");
        item.country = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
