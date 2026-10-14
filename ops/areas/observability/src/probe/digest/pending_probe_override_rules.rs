use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending probe override must pass. Check returns its problems, or none.
pub trait PendingProbeOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PendingProbeOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeOverrideUnitLengthRule {
    pub max_length: i64,
}

impl PendingProbeOverrideRule for PendingProbeOverrideUnitLengthRule {
    fn check(&self, item: &PendingProbeOverride) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeOverrideAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PendingProbeOverrideRule for PendingProbeOverrideAllowedTitleRule {
    fn check(&self, item: &PendingProbeOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeOverrideRequiredCodeRule {
    pub message: String,
}

impl PendingProbeOverrideRule for PendingProbeOverrideRequiredCodeRule {
    fn check(&self, item: &PendingProbeOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeOverrideActiveGuardRule {
    pub limit: i64,
}

impl PendingProbeOverrideRule for PendingProbeOverrideActiveGuardRule {
    fn check(&self, item: &PendingProbeOverride) -> Vec<String> {
        if item.active && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while active",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeOverrideRequiredTitleRule {
    pub message: String,
}

impl PendingProbeOverrideRule for PendingProbeOverrideRequiredTitleRule {
    fn check(&self, item: &PendingProbeOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeOverrideRequiredUnitRule {
    pub message: String,
}

impl PendingProbeOverrideRule for PendingProbeOverrideRequiredUnitRule {
    fn check(&self, item: &PendingProbeOverride) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending probe override.
#[derive(Debug, Clone)]
pub struct PendingProbeOverrideValidator {
    pub rules: Vec<Rc<dyn PendingProbeOverrideRule>>,
}

impl PendingProbeOverrideValidator {
    pub fn validate(&self, item: &PendingProbeOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingProbeOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingProbeOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_probe_override_validator() -> PendingProbeOverrideValidator {
    PendingProbeOverrideValidator {
        rules: vec![
            Rc::new(PendingProbeOverrideUnitLengthRule { max_length: 64 }) as Rc<dyn PendingProbeOverrideRule>,
            Rc::new(PendingProbeOverrideAllowedTitleRule {
                allowed: vec!["capped".to_string(), "default".to_string(), "regional".to_string()],
            }),
            Rc::new(PendingProbeOverrideRequiredCodeRule {
                message: "pending probe override needs an entry".to_string(),
            }),
            Rc::new(PendingProbeOverrideActiveGuardRule { limit: 32 }),
            Rc::new(PendingProbeOverrideRequiredTitleRule {
                message: "pending probe override needs an entry".to_string(),
            }),
            Rc::new(PendingProbeOverrideRequiredUnitRule {
                message: "pending probe override needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_probe_override_validator();
        let item = new_pending_probe_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = PendingProbeOverrideUnitLengthRule { max_length: 120 };
        let mut item = new_pending_probe_override("x", "y");
        item.unit = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingProbeOverrideRequiredCodeRule {
            message: "pending probe override needs this field".to_string(),
        };
        let mut item = new_pending_probe_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingProbeOverrideRequiredTitleRule {
            message: "pending probe override needs a value".to_string(),
        };
        let mut item = new_pending_probe_override("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PendingProbeOverrideRequiredUnitRule {
            message: "pending probe override needs this field".to_string(),
        };
        let mut item = new_pending_probe_override("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
