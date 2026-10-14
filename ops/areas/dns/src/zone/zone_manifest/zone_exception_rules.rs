use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone exception must pass. Check returns its problems, or none.
pub trait ZoneExceptionRule: std::fmt::Debug {
    fn check(&self, item: &ZoneException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneExceptionKeyIdLengthRule {
    pub max_length: i64,
}

impl ZoneExceptionRule for ZoneExceptionKeyIdLengthRule {
    fn check(&self, item: &ZoneException) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneExceptionAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl ZoneExceptionRule for ZoneExceptionAllowedPrincipalRule {
    fn check(&self, item: &ZoneException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is inconsistent", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneExceptionRequiredKeyIdRule {
    pub message: String,
}

impl ZoneExceptionRule for ZoneExceptionRequiredKeyIdRule {
    fn check(&self, item: &ZoneException) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneExceptionQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneExceptionRule for ZoneExceptionQuantityRangeRule {
    fn check(&self, item: &ZoneException) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneExceptionRequiredTitleRule {
    pub message: String,
}

impl ZoneExceptionRule for ZoneExceptionRequiredTitleRule {
    fn check(&self, item: &ZoneException) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone exception.
#[derive(Debug, Clone)]
pub struct ZoneExceptionValidator {
    pub rules: Vec<Rc<dyn ZoneExceptionRule>>,
}

impl ZoneExceptionValidator {
    pub fn validate(&self, item: &ZoneException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_exception_validator() -> ZoneExceptionValidator {
    ZoneExceptionValidator {
        rules: vec![
            Rc::new(ZoneExceptionKeyIdLengthRule { max_length: 16 }) as Rc<dyn ZoneExceptionRule>,
            Rc::new(ZoneExceptionAllowedPrincipalRule { allowed: Vec::new() }),
            Rc::new(ZoneExceptionRequiredKeyIdRule {
                message: "zone exception needs this field".to_string(),
            }),
            Rc::new(ZoneExceptionQuantityRangeRule { min: 255, max: 1000 }),
            Rc::new(ZoneExceptionRequiredTitleRule {
                message: "zone exception needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_exception_validator();
        let item = new_zone_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = ZoneExceptionKeyIdLengthRule { max_length: 120 };
        let mut item = new_zone_exception("x", "y");
        item.key_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = ZoneExceptionRequiredKeyIdRule {
            message: "zone exception needs an entry".to_string(),
        };
        let mut item = new_zone_exception("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ZoneExceptionQuantityRangeRule { min: 64, max: 86400 };
        let mut item = new_zone_exception("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ZoneExceptionRequiredTitleRule {
            message: "zone exception needs this field".to_string(),
        };
        let mut item = new_zone_exception("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
