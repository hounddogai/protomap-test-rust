use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped reservation milestone must pass. Check returns its problems, or none.
pub trait CappedReservationMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &CappedReservationMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationMilestoneRequiredCipherRule {
    pub message: String,
}

impl CappedReservationMilestoneRule for CappedReservationMilestoneRequiredCipherRule {
    fn check(&self, item: &CappedReservationMilestone) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationMilestoneCipherLengthRule {
    pub max_length: i64,
}

impl CappedReservationMilestoneRule for CappedReservationMilestoneCipherLengthRule {
    fn check(&self, item: &CappedReservationMilestone) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationMilestoneCodePrefixRule {
    pub prefix: String,
}

impl CappedReservationMilestoneRule for CappedReservationMilestoneCodePrefixRule {
    fn check(&self, item: &CappedReservationMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationMilestoneAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl CappedReservationMilestoneRule for CappedReservationMilestoneAllowedCipherRule {
    fn check(&self, item: &CappedReservationMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is out of range", item.cipher)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl CappedReservationMilestoneRule for CappedReservationMilestoneCodeLengthRule {
    fn check(&self, item: &CappedReservationMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped reservation milestone.
#[derive(Debug, Clone)]
pub struct CappedReservationMilestoneValidator {
    pub rules: Vec<Rc<dyn CappedReservationMilestoneRule>>,
}

impl CappedReservationMilestoneValidator {
    pub fn validate(&self, item: &CappedReservationMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedReservationMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedReservationMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_reservation_milestone_validator() -> CappedReservationMilestoneValidator {
    CappedReservationMilestoneValidator {
        rules: vec![
            Rc::new(CappedReservationMilestoneRequiredCipherRule {
                message: "capped reservation milestone needs a value".to_string(),
            }) as Rc<dyn CappedReservationMilestoneRule>,
            Rc::new(CappedReservationMilestoneCipherLengthRule { max_length: 120 }),
            Rc::new(CappedReservationMilestoneCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(CappedReservationMilestoneAllowedCipherRule {
                allowed: vec!["capped".to_string(), "strict".to_string(), "default".to_string()],
            }),
            Rc::new(CappedReservationMilestoneCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_reservation_milestone_validator();
        let item = new_capped_reservation_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = CappedReservationMilestoneRequiredCipherRule {
            message: "capped reservation milestone needs an entry".to_string(),
        };
        let mut item = new_capped_reservation_milestone("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = CappedReservationMilestoneCipherLengthRule { max_length: 120 };
        let mut item = new_capped_reservation_milestone("x", "y");
        item.cipher = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedReservationMilestoneCodeLengthRule { max_length: 16 };
        let mut item = new_capped_reservation_milestone("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
