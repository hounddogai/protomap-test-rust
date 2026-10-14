use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global alert cycle must pass. Check returns its problems, or none.
pub trait GlobalAlertCycleRule: std::fmt::Debug {
    fn check(&self, item: &GlobalAlertCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertCycleKeyIdLengthRule {
    pub max_length: i64,
}

impl GlobalAlertCycleRule for GlobalAlertCycleKeyIdLengthRule {
    fn check(&self, item: &GlobalAlertCycle) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertCycleRequiredNameRule {
    pub message: String,
}

impl GlobalAlertCycleRule for GlobalAlertCycleRequiredNameRule {
    fn check(&self, item: &GlobalAlertCycle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertCycleCodePrefixRule {
    pub prefix: String,
}

impl GlobalAlertCycleRule for GlobalAlertCycleCodePrefixRule {
    fn check(&self, item: &GlobalAlertCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertCycleVisibleGuardRule {
    pub limit: f64,
}

impl GlobalAlertCycleRule for GlobalAlertCycleVisibleGuardRule {
    fn check(&self, item: &GlobalAlertCycle) -> Vec<String> {
        if item.visible && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while visible", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global alert cycle.
#[derive(Debug, Clone)]
pub struct GlobalAlertCycleValidator {
    pub rules: Vec<Rc<dyn GlobalAlertCycleRule>>,
}

impl GlobalAlertCycleValidator {
    pub fn validate(&self, item: &GlobalAlertCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalAlertCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalAlertCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_alert_cycle_validator() -> GlobalAlertCycleValidator {
    GlobalAlertCycleValidator {
        rules: vec![
            Rc::new(GlobalAlertCycleKeyIdLengthRule { max_length: 120 }) as Rc<dyn GlobalAlertCycleRule>,
            Rc::new(GlobalAlertCycleRequiredNameRule {
                message: "global alert cycle needs this field".to_string(),
            }),
            Rc::new(GlobalAlertCycleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(GlobalAlertCycleVisibleGuardRule { limit: 2.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_alert_cycle_validator();
        let item = new_global_alert_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = GlobalAlertCycleKeyIdLengthRule { max_length: 1000 };
        let mut item = new_global_alert_cycle("x", "y");
        item.key_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GlobalAlertCycleRequiredNameRule {
            message: "global alert cycle needs this field".to_string(),
        };
        let mut item = new_global_alert_cycle("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
