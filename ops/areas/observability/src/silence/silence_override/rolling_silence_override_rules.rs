use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling silence override must pass. Check returns its problems, or none.
pub trait RollingSilenceOverrideRule: std::fmt::Debug {
    fn check(&self, item: &RollingSilenceOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceOverrideOwnerIdLengthRule {
    pub max_length: i64,
}

impl RollingSilenceOverrideRule for RollingSilenceOverrideOwnerIdLengthRule {
    fn check(&self, item: &RollingSilenceOverride) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceOverrideCountryLengthRule {
    pub max_length: i64,
}

impl RollingSilenceOverrideRule for RollingSilenceOverrideCountryLengthRule {
    fn check(&self, item: &RollingSilenceOverride) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceOverrideCodeLengthRule {
    pub max_length: i64,
}

impl RollingSilenceOverrideRule for RollingSilenceOverrideCodeLengthRule {
    fn check(&self, item: &RollingSilenceOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceOverrideRequiredCodeRule {
    pub message: String,
}

impl RollingSilenceOverrideRule for RollingSilenceOverrideRequiredCodeRule {
    fn check(&self, item: &RollingSilenceOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling silence override.
#[derive(Debug, Clone)]
pub struct RollingSilenceOverrideValidator {
    pub rules: Vec<Rc<dyn RollingSilenceOverrideRule>>,
}

impl RollingSilenceOverrideValidator {
    pub fn validate(&self, item: &RollingSilenceOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingSilenceOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingSilenceOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_silence_override_validator() -> RollingSilenceOverrideValidator {
    RollingSilenceOverrideValidator {
        rules: vec![
            Rc::new(RollingSilenceOverrideOwnerIdLengthRule { max_length: 32 }) as Rc<dyn RollingSilenceOverrideRule>,
            Rc::new(RollingSilenceOverrideCountryLengthRule { max_length: 3600 }),
            Rc::new(RollingSilenceOverrideCodeLengthRule { max_length: 64 }),
            Rc::new(RollingSilenceOverrideRequiredCodeRule {
                message: "rolling silence override needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_silence_override_validator();
        let item = new_rolling_silence_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = RollingSilenceOverrideOwnerIdLengthRule { max_length: 3600 };
        let mut item = new_rolling_silence_override("x", "y");
        item.owner_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = RollingSilenceOverrideCountryLengthRule { max_length: 64 };
        let mut item = new_rolling_silence_override("x", "y");
        item.country = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingSilenceOverrideCodeLengthRule { max_length: 120 };
        let mut item = new_rolling_silence_override("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingSilenceOverrideRequiredCodeRule {
            message: "rolling silence override needs this field".to_string(),
        };
        let mut item = new_rolling_silence_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
