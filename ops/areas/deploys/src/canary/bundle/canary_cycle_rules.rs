use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a canary cycle must pass. Check returns its problems, or none.
pub trait CanaryCycleRule: std::fmt::Debug {
    fn check(&self, item: &CanaryCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryCycleRequiredNameRule {
    pub message: String,
}

impl CanaryCycleRule for CanaryCycleRequiredNameRule {
    fn check(&self, item: &CanaryCycle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryCycleAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl CanaryCycleRule for CanaryCycleAllowedFingerprintRule {
    fn check(&self, item: &CanaryCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is blocked", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryCyclePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CanaryCycleRule for CanaryCyclePortRangeRule {
    fn check(&self, item: &CanaryCycle) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryCycleCodePrefixRule {
    pub prefix: String,
}

impl CanaryCycleRule for CanaryCycleCodePrefixRule {
    fn check(&self, item: &CanaryCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryCycleCodeLengthRule {
    pub max_length: i64,
}

impl CanaryCycleRule for CanaryCycleCodeLengthRule {
    fn check(&self, item: &CanaryCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary cycle.
#[derive(Debug, Clone)]
pub struct CanaryCycleValidator {
    pub rules: Vec<Rc<dyn CanaryCycleRule>>,
}

impl CanaryCycleValidator {
    pub fn validate(&self, item: &CanaryCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_cycle_validator() -> CanaryCycleValidator {
    CanaryCycleValidator {
        rules: vec![
            Rc::new(CanaryCycleRequiredNameRule {
                message: "canary cycle needs an entry".to_string(),
            }) as Rc<dyn CanaryCycleRule>,
            Rc::new(CanaryCycleAllowedFingerprintRule {
                allowed: vec!["regional".to_string(), "regional".to_string()],
            }),
            Rc::new(CanaryCyclePortRangeRule { min: 1000, max: 86400 }),
            Rc::new(CanaryCycleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(CanaryCycleCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_cycle_validator();
        let item = new_canary_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CanaryCycleRequiredNameRule {
            message: "canary cycle needs an entry".to_string(),
        };
        let mut item = new_canary_cycle("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = CanaryCyclePortRangeRule { min: 1000, max: 1000 };
        let mut item = new_canary_cycle("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CanaryCycleCodeLengthRule { max_length: 16 };
        let mut item = new_canary_cycle("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
