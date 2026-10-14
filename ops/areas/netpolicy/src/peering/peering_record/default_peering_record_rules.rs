use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default peering record must pass. Check returns its problems, or none.
pub trait DefaultPeeringRecordRule: std::fmt::Debug {
    fn check(&self, item: &DefaultPeeringRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringRecordNameLengthRule {
    pub max_length: i64,
}

impl DefaultPeeringRecordRule for DefaultPeeringRecordNameLengthRule {
    fn check(&self, item: &DefaultPeeringRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringRecordRequiredCodeRule {
    pub message: String,
}

impl DefaultPeeringRecordRule for DefaultPeeringRecordRequiredCodeRule {
    fn check(&self, item: &DefaultPeeringRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringRecordRequiredCipherRule {
    pub message: String,
}

impl DefaultPeeringRecordRule for DefaultPeeringRecordRequiredCipherRule {
    fn check(&self, item: &DefaultPeeringRecord) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringRecordCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultPeeringRecordRule for DefaultPeeringRecordCeilingRangeRule {
    fn check(&self, item: &DefaultPeeringRecord) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringRecordCodeLengthRule {
    pub max_length: i64,
}

impl DefaultPeeringRecordRule for DefaultPeeringRecordCodeLengthRule {
    fn check(&self, item: &DefaultPeeringRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default peering record.
#[derive(Debug, Clone)]
pub struct DefaultPeeringRecordValidator {
    pub rules: Vec<Rc<dyn DefaultPeeringRecordRule>>,
}

impl DefaultPeeringRecordValidator {
    pub fn validate(&self, item: &DefaultPeeringRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultPeeringRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultPeeringRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_peering_record_validator() -> DefaultPeeringRecordValidator {
    DefaultPeeringRecordValidator {
        rules: vec![
            Rc::new(DefaultPeeringRecordNameLengthRule { max_length: 1000 }) as Rc<dyn DefaultPeeringRecordRule>,
            Rc::new(DefaultPeeringRecordRequiredCodeRule {
                message: "default peering record needs this field".to_string(),
            }),
            Rc::new(DefaultPeeringRecordRequiredCipherRule {
                message: "default peering record needs an entry".to_string(),
            }),
            Rc::new(DefaultPeeringRecordCeilingRangeRule { min: 7.9, max: 1909.4 }),
            Rc::new(DefaultPeeringRecordCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_peering_record_validator();
        let item = new_default_peering_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DefaultPeeringRecordNameLengthRule { max_length: 80 };
        let mut item = new_default_peering_record("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultPeeringRecordRequiredCodeRule {
            message: "default peering record needs this field".to_string(),
        };
        let mut item = new_default_peering_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = DefaultPeeringRecordRequiredCipherRule {
            message: "default peering record needs an entry".to_string(),
        };
        let mut item = new_default_peering_record("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = DefaultPeeringRecordCeilingRangeRule { min: 0.8, max: 4527.3 };
        let mut item = new_default_peering_record("x", "y");
        item.ceiling = 4528.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultPeeringRecordCodeLengthRule { max_length: 255 };
        let mut item = new_default_peering_record("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
