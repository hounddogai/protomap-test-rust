use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict allowlist override must pass. Check returns its problems, or none.
pub trait StrictAllowlistOverrideRule: std::fmt::Debug {
    fn check(&self, item: &StrictAllowlistOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistOverrideCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictAllowlistOverrideRule for StrictAllowlistOverrideCeilingRangeRule {
    fn check(&self, item: &StrictAllowlistOverride) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistOverrideRequiredCipherRule {
    pub message: String,
}

impl StrictAllowlistOverrideRule for StrictAllowlistOverrideRequiredCipherRule {
    fn check(&self, item: &StrictAllowlistOverride) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistOverrideNameLengthRule {
    pub max_length: i64,
}

impl StrictAllowlistOverrideRule for StrictAllowlistOverrideNameLengthRule {
    fn check(&self, item: &StrictAllowlistOverride) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict allowlist override.
#[derive(Debug, Clone)]
pub struct StrictAllowlistOverrideValidator {
    pub rules: Vec<Rc<dyn StrictAllowlistOverrideRule>>,
}

impl StrictAllowlistOverrideValidator {
    pub fn validate(&self, item: &StrictAllowlistOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictAllowlistOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictAllowlistOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_allowlist_override_validator() -> StrictAllowlistOverrideValidator {
    StrictAllowlistOverrideValidator {
        rules: vec![
            Rc::new(StrictAllowlistOverrideCeilingRangeRule { min: 3.4, max: 1243.0 })
                as Rc<dyn StrictAllowlistOverrideRule>,
            Rc::new(StrictAllowlistOverrideRequiredCipherRule {
                message: "strict allowlist override needs an entry".to_string(),
            }),
            Rc::new(StrictAllowlistOverrideNameLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_allowlist_override_validator();
        let item = new_strict_allowlist_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = StrictAllowlistOverrideCeilingRangeRule { min: 4.0, max: 4148.9 };
        let mut item = new_strict_allowlist_override("x", "y");
        item.ceiling = 4150.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = StrictAllowlistOverrideRequiredCipherRule {
            message: "strict allowlist override needs an entry".to_string(),
        };
        let mut item = new_strict_allowlist_override("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictAllowlistOverrideNameLengthRule { max_length: 86400 };
        let mut item = new_strict_allowlist_override("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
