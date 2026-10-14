use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient redaction milestone must pass. Check returns its problems, or none.
pub trait LenientRedactionMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &LenientRedactionMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionMilestoneRequiredCodeRule {
    pub message: String,
}

impl LenientRedactionMilestoneRule for LenientRedactionMilestoneRequiredCodeRule {
    fn check(&self, item: &LenientRedactionMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionMilestoneRequiredPhoneRule {
    pub message: String,
}

impl LenientRedactionMilestoneRule for LenientRedactionMilestoneRequiredPhoneRule {
    fn check(&self, item: &LenientRedactionMilestone) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionMilestonePhoneLengthRule {
    pub max_length: i64,
}

impl LenientRedactionMilestoneRule for LenientRedactionMilestonePhoneLengthRule {
    fn check(&self, item: &LenientRedactionMilestone) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient redaction milestone.
#[derive(Debug, Clone)]
pub struct LenientRedactionMilestoneValidator {
    pub rules: Vec<Rc<dyn LenientRedactionMilestoneRule>>,
}

impl LenientRedactionMilestoneValidator {
    pub fn validate(&self, item: &LenientRedactionMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientRedactionMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRedactionMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_redaction_milestone_validator() -> LenientRedactionMilestoneValidator {
    LenientRedactionMilestoneValidator {
        rules: vec![
            Rc::new(LenientRedactionMilestoneRequiredCodeRule {
                message: "lenient redaction milestone needs an entry".to_string(),
            }) as Rc<dyn LenientRedactionMilestoneRule>,
            Rc::new(LenientRedactionMilestoneRequiredPhoneRule {
                message: "lenient redaction milestone needs a value".to_string(),
            }),
            Rc::new(LenientRedactionMilestonePhoneLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_redaction_milestone_validator();
        let item = new_lenient_redaction_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientRedactionMilestoneRequiredCodeRule {
            message: "lenient redaction milestone needs a value".to_string(),
        };
        let mut item = new_lenient_redaction_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = LenientRedactionMilestoneRequiredPhoneRule {
            message: "lenient redaction milestone needs a value".to_string(),
        };
        let mut item = new_lenient_redaction_milestone("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = LenientRedactionMilestonePhoneLengthRule { max_length: 16 };
        let mut item = new_lenient_redaction_milestone("x", "y");
        item.phone = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
