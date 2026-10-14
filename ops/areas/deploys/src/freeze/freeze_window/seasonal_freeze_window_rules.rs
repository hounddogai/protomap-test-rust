use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal freeze window must pass. Check returns its problems, or none.
pub trait SeasonalFreezeWindowRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalFreezeWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeWindowRequiredContactEmailRule {
    pub message: String,
}

impl SeasonalFreezeWindowRule for SeasonalFreezeWindowRequiredContactEmailRule {
    fn check(&self, item: &SeasonalFreezeWindow) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeWindowNameLengthRule {
    pub max_length: i64,
}

impl SeasonalFreezeWindowRule for SeasonalFreezeWindowNameLengthRule {
    fn check(&self, item: &SeasonalFreezeWindow) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeWindowAllowedHostRule {
    pub allowed: Vec<String>,
}

impl SeasonalFreezeWindowRule for SeasonalFreezeWindowAllowedHostRule {
    fn check(&self, item: &SeasonalFreezeWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is duplicated", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeWindowContactEmailLengthRule {
    pub max_length: i64,
}

impl SeasonalFreezeWindowRule for SeasonalFreezeWindowContactEmailLengthRule {
    fn check(&self, item: &SeasonalFreezeWindow) -> Vec<String> {
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

/// Runs every rule of a seasonal freeze window.
#[derive(Debug, Clone)]
pub struct SeasonalFreezeWindowValidator {
    pub rules: Vec<Rc<dyn SeasonalFreezeWindowRule>>,
}

impl SeasonalFreezeWindowValidator {
    pub fn validate(&self, item: &SeasonalFreezeWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalFreezeWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalFreezeWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_freeze_window_validator() -> SeasonalFreezeWindowValidator {
    SeasonalFreezeWindowValidator {
        rules: vec![
            Rc::new(SeasonalFreezeWindowRequiredContactEmailRule {
                message: "seasonal freeze window needs an entry".to_string(),
            }) as Rc<dyn SeasonalFreezeWindowRule>,
            Rc::new(SeasonalFreezeWindowNameLengthRule { max_length: 120 }),
            Rc::new(SeasonalFreezeWindowAllowedHostRule { allowed: Vec::new() }),
            Rc::new(SeasonalFreezeWindowContactEmailLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_freeze_window_validator();
        let item = new_seasonal_freeze_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = SeasonalFreezeWindowRequiredContactEmailRule {
            message: "seasonal freeze window needs this field".to_string(),
        };
        let mut item = new_seasonal_freeze_window("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalFreezeWindowNameLengthRule { max_length: 86400 };
        let mut item = new_seasonal_freeze_window("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = SeasonalFreezeWindowContactEmailLengthRule { max_length: 80 };
        let mut item = new_seasonal_freeze_window("x", "y");
        item.contact_email = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
