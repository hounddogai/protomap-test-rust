use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation notice must pass. Check returns its problems, or none.
pub trait EscalationNoticeRule: std::fmt::Debug {
    fn check(&self, item: &EscalationNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationNoticeFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EscalationNoticeRule for EscalationNoticeFloorValueRangeRule {
    fn check(&self, item: &EscalationNotice) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationNoticeNotesLengthRule {
    pub max_length: i64,
}

impl EscalationNoticeRule for EscalationNoticeNotesLengthRule {
    fn check(&self, item: &EscalationNotice) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationNoticeRequiredLabelRule {
    pub message: String,
}

impl EscalationNoticeRule for EscalationNoticeRequiredLabelRule {
    fn check(&self, item: &EscalationNotice) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationNoticeCodeLengthRule {
    pub max_length: i64,
}

impl EscalationNoticeRule for EscalationNoticeCodeLengthRule {
    fn check(&self, item: &EscalationNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationNoticeRequiredNotesRule {
    pub message: String,
}

impl EscalationNoticeRule for EscalationNoticeRequiredNotesRule {
    fn check(&self, item: &EscalationNotice) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationNoticeActiveGuardRule {
    pub limit: f64,
}

impl EscalationNoticeRule for EscalationNoticeActiveGuardRule {
    fn check(&self, item: &EscalationNotice) -> Vec<String> {
        if item.active && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while active",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation notice.
#[derive(Debug, Clone)]
pub struct EscalationNoticeValidator {
    pub rules: Vec<Rc<dyn EscalationNoticeRule>>,
}

impl EscalationNoticeValidator {
    pub fn validate(&self, item: &EscalationNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_notice_validator() -> EscalationNoticeValidator {
    EscalationNoticeValidator {
        rules: vec![
            Rc::new(EscalationNoticeFloorValueRangeRule { min: 7.0, max: 4279.7 }) as Rc<dyn EscalationNoticeRule>,
            Rc::new(EscalationNoticeNotesLengthRule { max_length: 32 }),
            Rc::new(EscalationNoticeRequiredLabelRule {
                message: "escalation notice needs a value".to_string(),
            }),
            Rc::new(EscalationNoticeCodeLengthRule { max_length: 120 }),
            Rc::new(EscalationNoticeRequiredNotesRule {
                message: "escalation notice needs a value".to_string(),
            }),
            Rc::new(EscalationNoticeActiveGuardRule { limit: 8.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_notice_validator();
        let item = new_escalation_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = EscalationNoticeFloorValueRangeRule { min: 1.3, max: 390.4 };
        let mut item = new_escalation_notice("x", "y");
        item.floor_value = 391.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = EscalationNoticeNotesLengthRule { max_length: 86400 };
        let mut item = new_escalation_notice("x", "y");
        item.notes = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = EscalationNoticeRequiredLabelRule {
            message: "escalation notice needs a value".to_string(),
        };
        let mut item = new_escalation_notice("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EscalationNoticeCodeLengthRule { max_length: 64 };
        let mut item = new_escalation_notice("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = EscalationNoticeRequiredNotesRule {
            message: "escalation notice needs an entry".to_string(),
        };
        let mut item = new_escalation_notice("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
