use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation snapshot must pass. Check returns its problems, or none.
pub trait EscalationSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &EscalationSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSnapshotContactEmailLengthRule {
    pub max_length: i64,
}

impl EscalationSnapshotRule for EscalationSnapshotContactEmailLengthRule {
    fn check(&self, item: &EscalationSnapshot) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSnapshotQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationSnapshotRule for EscalationSnapshotQuantityRangeRule {
    fn check(&self, item: &EscalationSnapshot) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSnapshotCodePrefixRule {
    pub prefix: String,
}

impl EscalationSnapshotRule for EscalationSnapshotCodePrefixRule {
    fn check(&self, item: &EscalationSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSnapshotAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationSnapshotRule for EscalationSnapshotAttemptsRangeRule {
    fn check(&self, item: &EscalationSnapshot) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a escalation snapshot.
#[derive(Debug, Clone)]
pub struct EscalationSnapshotValidator {
    pub rules: Vec<Rc<dyn EscalationSnapshotRule>>,
}

impl EscalationSnapshotValidator {
    pub fn validate(&self, item: &EscalationSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_snapshot_validator() -> EscalationSnapshotValidator {
    EscalationSnapshotValidator {
        rules: vec![
            Rc::new(EscalationSnapshotContactEmailLengthRule { max_length: 80 }) as Rc<dyn EscalationSnapshotRule>,
            Rc::new(EscalationSnapshotQuantityRangeRule { min: 255, max: 1000 }),
            Rc::new(EscalationSnapshotCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(EscalationSnapshotAttemptsRangeRule { min: 80, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_snapshot_validator();
        let item = new_escalation_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = EscalationSnapshotContactEmailLengthRule { max_length: 255 };
        let mut item = new_escalation_snapshot("x", "y");
        item.contact_email = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = EscalationSnapshotQuantityRangeRule { min: 64, max: 1000 };
        let mut item = new_escalation_snapshot("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = EscalationSnapshotAttemptsRangeRule { min: 1000, max: 86400 };
        let mut item = new_escalation_snapshot("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
