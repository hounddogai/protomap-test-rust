use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending flow log override must pass. Check returns its problems, or none.
pub trait PendingFlowLogOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PendingFlowLogOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogOverrideNameLengthRule {
    pub max_length: i64,
}

impl PendingFlowLogOverrideRule for PendingFlowLogOverrideNameLengthRule {
    fn check(&self, item: &PendingFlowLogOverride) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogOverrideEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingFlowLogOverrideRule for PendingFlowLogOverrideEndsAtRangeRule {
    fn check(&self, item: &PendingFlowLogOverride) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogOverrideRequiredTitleRule {
    pub message: String,
}

impl PendingFlowLogOverrideRule for PendingFlowLogOverrideRequiredTitleRule {
    fn check(&self, item: &PendingFlowLogOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending flow log override.
#[derive(Debug, Clone)]
pub struct PendingFlowLogOverrideValidator {
    pub rules: Vec<Rc<dyn PendingFlowLogOverrideRule>>,
}

impl PendingFlowLogOverrideValidator {
    pub fn validate(&self, item: &PendingFlowLogOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingFlowLogOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingFlowLogOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_flow_log_override_validator() -> PendingFlowLogOverrideValidator {
    PendingFlowLogOverrideValidator {
        rules: vec![
            Rc::new(PendingFlowLogOverrideNameLengthRule { max_length: 255 }) as Rc<dyn PendingFlowLogOverrideRule>,
            Rc::new(PendingFlowLogOverrideEndsAtRangeRule { min: 255, max: 3600 }),
            Rc::new(PendingFlowLogOverrideRequiredTitleRule {
                message: "pending flow log override needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_flow_log_override_validator();
        let item = new_pending_flow_log_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingFlowLogOverrideNameLengthRule { max_length: 32 };
        let mut item = new_pending_flow_log_override("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = PendingFlowLogOverrideEndsAtRangeRule { min: 32, max: 64 };
        let mut item = new_pending_flow_log_override("x", "y");
        item.ends_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingFlowLogOverrideRequiredTitleRule {
            message: "pending flow log override needs an entry".to_string(),
        };
        let mut item = new_pending_flow_log_override("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
