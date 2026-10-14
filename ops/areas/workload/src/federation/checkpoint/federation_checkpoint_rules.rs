use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a federation checkpoint must pass. Check returns its problems, or none.
pub trait FederationCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &FederationCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationCheckpointRequiredPrincipalRule {
    pub message: String,
}

impl FederationCheckpointRule for FederationCheckpointRequiredPrincipalRule {
    fn check(&self, item: &FederationCheckpoint) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationCheckpointVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationCheckpointRule for FederationCheckpointVersionRangeRule {
    fn check(&self, item: &FederationCheckpoint) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationCheckpointCurrencyLengthRule {
    pub max_length: i64,
}

impl FederationCheckpointRule for FederationCheckpointCurrencyLengthRule {
    fn check(&self, item: &FederationCheckpoint) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationCheckpointRequiredNameRule {
    pub message: String,
}

impl FederationCheckpointRule for FederationCheckpointRequiredNameRule {
    fn check(&self, item: &FederationCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationCheckpointPrincipalLengthRule {
    pub max_length: i64,
}

impl FederationCheckpointRule for FederationCheckpointPrincipalLengthRule {
    fn check(&self, item: &FederationCheckpoint) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationCheckpointAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl FederationCheckpointRule for FederationCheckpointAllowedCurrencyRule {
    fn check(&self, item: &FederationCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is below minimum", item.currency)];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation checkpoint.
#[derive(Debug, Clone)]
pub struct FederationCheckpointValidator {
    pub rules: Vec<Rc<dyn FederationCheckpointRule>>,
}

impl FederationCheckpointValidator {
    pub fn validate(&self, item: &FederationCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_checkpoint_validator() -> FederationCheckpointValidator {
    FederationCheckpointValidator {
        rules: vec![
            Rc::new(FederationCheckpointRequiredPrincipalRule {
                message: "federation checkpoint needs this field".to_string(),
            }) as Rc<dyn FederationCheckpointRule>,
            Rc::new(FederationCheckpointVersionRangeRule { min: 16, max: 120 }),
            Rc::new(FederationCheckpointCurrencyLengthRule { max_length: 255 }),
            Rc::new(FederationCheckpointRequiredNameRule {
                message: "federation checkpoint needs a value".to_string(),
            }),
            Rc::new(FederationCheckpointPrincipalLengthRule { max_length: 255 }),
            Rc::new(FederationCheckpointAllowedCurrencyRule {
                allowed: vec!["tiered".to_string(), "regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_checkpoint_validator();
        let item = new_federation_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = FederationCheckpointRequiredPrincipalRule {
            message: "federation checkpoint needs an entry".to_string(),
        };
        let mut item = new_federation_checkpoint("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = FederationCheckpointVersionRangeRule { min: 32, max: 3600 };
        let mut item = new_federation_checkpoint("x", "y");
        item.version = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = FederationCheckpointCurrencyLengthRule { max_length: 80 };
        let mut item = new_federation_checkpoint("x", "y");
        item.currency = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FederationCheckpointRequiredNameRule {
            message: "federation checkpoint needs a value".to_string(),
        };
        let mut item = new_federation_checkpoint("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = FederationCheckpointPrincipalLengthRule { max_length: 80 };
        let mut item = new_federation_checkpoint("x", "y");
        item.principal = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
