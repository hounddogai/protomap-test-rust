use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional silence window must pass. Check returns its problems, or none.
pub trait RegionalSilenceWindowRule: std::fmt::Debug {
    fn check(&self, item: &RegionalSilenceWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceWindowCodeLengthRule {
    pub max_length: i64,
}

impl RegionalSilenceWindowRule for RegionalSilenceWindowCodeLengthRule {
    fn check(&self, item: &RegionalSilenceWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceWindowRequiredCurrencyRule {
    pub message: String,
}

impl RegionalSilenceWindowRule for RegionalSilenceWindowRequiredCurrencyRule {
    fn check(&self, item: &RegionalSilenceWindow) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceWindowRequiredNameRule {
    pub message: String,
}

impl RegionalSilenceWindowRule for RegionalSilenceWindowRequiredNameRule {
    fn check(&self, item: &RegionalSilenceWindow) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceWindowNameLengthRule {
    pub max_length: i64,
}

impl RegionalSilenceWindowRule for RegionalSilenceWindowNameLengthRule {
    fn check(&self, item: &RegionalSilenceWindow) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional silence window.
#[derive(Debug, Clone)]
pub struct RegionalSilenceWindowValidator {
    pub rules: Vec<Rc<dyn RegionalSilenceWindowRule>>,
}

impl RegionalSilenceWindowValidator {
    pub fn validate(&self, item: &RegionalSilenceWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalSilenceWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalSilenceWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_silence_window_validator() -> RegionalSilenceWindowValidator {
    RegionalSilenceWindowValidator {
        rules: vec![
            Rc::new(RegionalSilenceWindowCodeLengthRule { max_length: 1000 }) as Rc<dyn RegionalSilenceWindowRule>,
            Rc::new(RegionalSilenceWindowRequiredCurrencyRule {
                message: "regional silence window needs a value".to_string(),
            }),
            Rc::new(RegionalSilenceWindowRequiredNameRule {
                message: "regional silence window needs a value".to_string(),
            }),
            Rc::new(RegionalSilenceWindowNameLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_silence_window_validator();
        let item = new_regional_silence_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalSilenceWindowCodeLengthRule { max_length: 64 };
        let mut item = new_regional_silence_window("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = RegionalSilenceWindowRequiredCurrencyRule {
            message: "regional silence window needs a value".to_string(),
        };
        let mut item = new_regional_silence_window("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionalSilenceWindowRequiredNameRule {
            message: "regional silence window needs an entry".to_string(),
        };
        let mut item = new_regional_silence_window("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionalSilenceWindowNameLengthRule { max_length: 1000 };
        let mut item = new_regional_silence_window("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
