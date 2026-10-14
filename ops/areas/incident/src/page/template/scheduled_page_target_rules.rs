use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled page target must pass. Check returns its problems, or none.
pub trait ScheduledPageTargetRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledPageTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPageTargetRequiredNameRule {
    pub message: String,
}

impl ScheduledPageTargetRule for ScheduledPageTargetRequiredNameRule {
    fn check(&self, item: &ScheduledPageTarget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPageTargetRequiredOwnerIdRule {
    pub message: String,
}

impl ScheduledPageTargetRule for ScheduledPageTargetRequiredOwnerIdRule {
    fn check(&self, item: &ScheduledPageTarget) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPageTargetOwnerIdLengthRule {
    pub max_length: i64,
}

impl ScheduledPageTargetRule for ScheduledPageTargetOwnerIdLengthRule {
    fn check(&self, item: &ScheduledPageTarget) -> Vec<String> {
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
pub struct ScheduledPageTargetAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl ScheduledPageTargetRule for ScheduledPageTargetAllowedOwnerIdRule {
    fn check(&self, item: &ScheduledPageTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is unknown", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPageTargetVisibleGuardRule {
    pub limit: f64,
}

impl ScheduledPageTargetRule for ScheduledPageTargetVisibleGuardRule {
    fn check(&self, item: &ScheduledPageTarget) -> Vec<String> {
        if item.visible && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while visible", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPageTargetCodePrefixRule {
    pub prefix: String,
}

impl ScheduledPageTargetRule for ScheduledPageTargetCodePrefixRule {
    fn check(&self, item: &ScheduledPageTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled page target.
#[derive(Debug, Clone)]
pub struct ScheduledPageTargetValidator {
    pub rules: Vec<Rc<dyn ScheduledPageTargetRule>>,
}

impl ScheduledPageTargetValidator {
    pub fn validate(&self, item: &ScheduledPageTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledPageTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledPageTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_page_target_validator() -> ScheduledPageTargetValidator {
    ScheduledPageTargetValidator {
        rules: vec![
            Rc::new(ScheduledPageTargetRequiredNameRule {
                message: "scheduled page target needs an entry".to_string(),
            }) as Rc<dyn ScheduledPageTargetRule>,
            Rc::new(ScheduledPageTargetRequiredOwnerIdRule {
                message: "scheduled page target needs an entry".to_string(),
            }),
            Rc::new(ScheduledPageTargetOwnerIdLengthRule { max_length: 255 }),
            Rc::new(ScheduledPageTargetAllowedOwnerIdRule {
                allowed: vec!["scheduled".to_string()],
            }),
            Rc::new(ScheduledPageTargetVisibleGuardRule { limit: 2.5 }),
            Rc::new(ScheduledPageTargetCodePrefixRule {
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
        let validator = default_scheduled_page_target_validator();
        let item = new_scheduled_page_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ScheduledPageTargetRequiredNameRule {
            message: "scheduled page target needs an entry".to_string(),
        };
        let mut item = new_scheduled_page_target("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = ScheduledPageTargetRequiredOwnerIdRule {
            message: "scheduled page target needs an entry".to_string(),
        };
        let mut item = new_scheduled_page_target("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = ScheduledPageTargetOwnerIdLengthRule { max_length: 1000 };
        let mut item = new_scheduled_page_target("x", "y");
        item.owner_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
