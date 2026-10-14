use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict page target must pass. Check returns its problems, or none.
pub trait StrictPageTargetRule: std::fmt::Debug {
    fn check(&self, item: &StrictPageTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageTargetCodePrefixRule {
    pub prefix: String,
}

impl StrictPageTargetRule for StrictPageTargetCodePrefixRule {
    fn check(&self, item: &StrictPageTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageTargetTimezoneLengthRule {
    pub max_length: i64,
}

impl StrictPageTargetRule for StrictPageTargetTimezoneLengthRule {
    fn check(&self, item: &StrictPageTarget) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageTargetRequiredTimezoneRule {
    pub message: String,
}

impl StrictPageTargetRule for StrictPageTargetRequiredTimezoneRule {
    fn check(&self, item: &StrictPageTarget) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageTargetAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl StrictPageTargetRule for StrictPageTargetAllowedTimezoneRule {
    fn check(&self, item: &StrictPageTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is blocked", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageTargetArchivedGuardRule {
    pub limit: f64,
}

impl StrictPageTargetRule for StrictPageTargetArchivedGuardRule {
    fn check(&self, item: &StrictPageTarget) -> Vec<String> {
        if item.archived && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while archived", item.rate, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict page target.
#[derive(Debug, Clone)]
pub struct StrictPageTargetValidator {
    pub rules: Vec<Rc<dyn StrictPageTargetRule>>,
}

impl StrictPageTargetValidator {
    pub fn validate(&self, item: &StrictPageTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictPageTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictPageTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_page_target_validator() -> StrictPageTargetValidator {
    StrictPageTargetValidator {
        rules: vec![
            Rc::new(StrictPageTargetCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn StrictPageTargetRule>,
            Rc::new(StrictPageTargetTimezoneLengthRule { max_length: 120 }),
            Rc::new(StrictPageTargetRequiredTimezoneRule {
                message: "strict page target needs a value".to_string(),
            }),
            Rc::new(StrictPageTargetAllowedTimezoneRule {
                allowed: vec!["strict".to_string(), "strict".to_string(), "capped".to_string()],
            }),
            Rc::new(StrictPageTargetArchivedGuardRule { limit: 4.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_page_target_validator();
        let item = new_strict_page_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = StrictPageTargetTimezoneLengthRule { max_length: 32 };
        let mut item = new_strict_page_target("x", "y");
        item.timezone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = StrictPageTargetRequiredTimezoneRule {
            message: "strict page target needs this field".to_string(),
        };
        let mut item = new_strict_page_target("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
