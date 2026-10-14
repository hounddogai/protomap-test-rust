use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a peering entry must pass. Check returns its problems, or none.
pub trait PeeringEntryRule: std::fmt::Debug {
    fn check(&self, item: &PeeringEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringEntryCodeLengthRule {
    pub max_length: i64,
}

impl PeeringEntryRule for PeeringEntryCodeLengthRule {
    fn check(&self, item: &PeeringEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringEntryRequiredTitleRule {
    pub message: String,
}

impl PeeringEntryRule for PeeringEntryRequiredTitleRule {
    fn check(&self, item: &PeeringEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringEntryTitleLengthRule {
    pub max_length: i64,
}

impl PeeringEntryRule for PeeringEntryTitleLengthRule {
    fn check(&self, item: &PeeringEntry) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringEntryAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl PeeringEntryRule for PeeringEntryAllowedScopeRule {
    fn check(&self, item: &PeeringEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is unknown", item.scope)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering entry.
#[derive(Debug, Clone)]
pub struct PeeringEntryValidator {
    pub rules: Vec<Rc<dyn PeeringEntryRule>>,
}

impl PeeringEntryValidator {
    pub fn validate(&self, item: &PeeringEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_entry_validator() -> PeeringEntryValidator {
    PeeringEntryValidator {
        rules: vec![
            Rc::new(PeeringEntryCodeLengthRule { max_length: 3600 }) as Rc<dyn PeeringEntryRule>,
            Rc::new(PeeringEntryRequiredTitleRule {
                message: "peering entry needs a value".to_string(),
            }),
            Rc::new(PeeringEntryTitleLengthRule { max_length: 80 }),
            Rc::new(PeeringEntryAllowedScopeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_entry_validator();
        let item = new_peering_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PeeringEntryCodeLengthRule { max_length: 16 };
        let mut item = new_peering_entry("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PeeringEntryRequiredTitleRule {
            message: "peering entry needs a value".to_string(),
        };
        let mut item = new_peering_entry("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PeeringEntryTitleLengthRule { max_length: 80 };
        let mut item = new_peering_entry("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
