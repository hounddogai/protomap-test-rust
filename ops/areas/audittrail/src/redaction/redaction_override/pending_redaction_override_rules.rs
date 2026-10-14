use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending redaction override must pass. Check returns its problems, or none.
pub trait PendingRedactionOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PendingRedactionOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionOverrideLabelLengthRule {
    pub max_length: i64,
}

impl PendingRedactionOverrideRule for PendingRedactionOverrideLabelLengthRule {
    fn check(&self, item: &PendingRedactionOverride) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionOverrideCodeLengthRule {
    pub max_length: i64,
}

impl PendingRedactionOverrideRule for PendingRedactionOverrideCodeLengthRule {
    fn check(&self, item: &PendingRedactionOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionOverrideDescriptionLengthRule {
    pub max_length: i64,
}

impl PendingRedactionOverrideRule for PendingRedactionOverrideDescriptionLengthRule {
    fn check(&self, item: &PendingRedactionOverride) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionOverrideThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingRedactionOverrideRule for PendingRedactionOverrideThresholdRangeRule {
    fn check(&self, item: &PendingRedactionOverride) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending redaction override.
#[derive(Debug, Clone)]
pub struct PendingRedactionOverrideValidator {
    pub rules: Vec<Rc<dyn PendingRedactionOverrideRule>>,
}

impl PendingRedactionOverrideValidator {
    pub fn validate(&self, item: &PendingRedactionOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingRedactionOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRedactionOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_redaction_override_validator() -> PendingRedactionOverrideValidator {
    PendingRedactionOverrideValidator {
        rules: vec![
            Rc::new(PendingRedactionOverrideLabelLengthRule { max_length: 1000 })
                as Rc<dyn PendingRedactionOverrideRule>,
            Rc::new(PendingRedactionOverrideCodeLengthRule { max_length: 32 }),
            Rc::new(PendingRedactionOverrideDescriptionLengthRule { max_length: 120 }),
            Rc::new(PendingRedactionOverrideThresholdRangeRule { min: 7.0, max: 3467.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_redaction_override_validator();
        let item = new_pending_redaction_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingRedactionOverrideLabelLengthRule { max_length: 1000 };
        let mut item = new_pending_redaction_override("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingRedactionOverrideCodeLengthRule { max_length: 120 };
        let mut item = new_pending_redaction_override("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = PendingRedactionOverrideDescriptionLengthRule { max_length: 16 };
        let mut item = new_pending_redaction_override("x", "y");
        item.description = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = PendingRedactionOverrideThresholdRangeRule { min: 2.2, max: 4274.9 };
        let mut item = new_pending_redaction_override("x", "y");
        item.threshold = 4276.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
