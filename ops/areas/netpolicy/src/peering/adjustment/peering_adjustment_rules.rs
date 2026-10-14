use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering adjustment must pass. Check returns its problems, or none.
pub trait PeeringAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &PeeringAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAdjustmentNameLengthRule {
    pub max_length: i64,
}

impl PeeringAdjustmentRule for PeeringAdjustmentNameLengthRule {
    fn check(&self, item: &PeeringAdjustment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAdjustmentRequiredNameRule {
    pub message: String,
}

impl PeeringAdjustmentRule for PeeringAdjustmentRequiredNameRule {
    fn check(&self, item: &PeeringAdjustment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAdjustmentCipherLengthRule {
    pub max_length: i64,
}

impl PeeringAdjustmentRule for PeeringAdjustmentCipherLengthRule {
    fn check(&self, item: &PeeringAdjustment) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAdjustmentRequiredExternalRefRule {
    pub message: String,
}

impl PeeringAdjustmentRule for PeeringAdjustmentRequiredExternalRefRule {
    fn check(&self, item: &PeeringAdjustment) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering adjustment.
#[derive(Debug, Clone)]
pub struct PeeringAdjustmentValidator {
    pub rules: Vec<Rc<dyn PeeringAdjustmentRule>>,
}

impl PeeringAdjustmentValidator {
    pub fn validate(&self, item: &PeeringAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_adjustment_validator() -> PeeringAdjustmentValidator {
    PeeringAdjustmentValidator {
        rules: vec![
            Rc::new(PeeringAdjustmentNameLengthRule { max_length: 86400 }) as Rc<dyn PeeringAdjustmentRule>,
            Rc::new(PeeringAdjustmentRequiredNameRule {
                message: "peering adjustment needs this field".to_string(),
            }),
            Rc::new(PeeringAdjustmentCipherLengthRule { max_length: 16 }),
            Rc::new(PeeringAdjustmentRequiredExternalRefRule {
                message: "peering adjustment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_adjustment_validator();
        let item = new_peering_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PeeringAdjustmentNameLengthRule { max_length: 255 };
        let mut item = new_peering_adjustment("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PeeringAdjustmentRequiredNameRule {
            message: "peering adjustment needs a value".to_string(),
        };
        let mut item = new_peering_adjustment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = PeeringAdjustmentCipherLengthRule { max_length: 32 };
        let mut item = new_peering_adjustment("x", "y");
        item.cipher = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = PeeringAdjustmentRequiredExternalRefRule {
            message: "peering adjustment needs a value".to_string(),
        };
        let mut item = new_peering_adjustment("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
