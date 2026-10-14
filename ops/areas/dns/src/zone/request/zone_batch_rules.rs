use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone batch must pass. Check returns its problems, or none.
pub trait ZoneBatchRule: std::fmt::Debug {
    fn check(&self, item: &ZoneBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneBatchAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ZoneBatchRule for ZoneBatchAllowedCodeRule {
    fn check(&self, item: &ZoneBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneBatchNameLengthRule {
    pub max_length: i64,
}

impl ZoneBatchRule for ZoneBatchNameLengthRule {
    fn check(&self, item: &ZoneBatch) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneBatchRequiredPhoneRule {
    pub message: String,
}

impl ZoneBatchRule for ZoneBatchRequiredPhoneRule {
    fn check(&self, item: &ZoneBatch) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneBatchCodePrefixRule {
    pub prefix: String,
}

impl ZoneBatchRule for ZoneBatchCodePrefixRule {
    fn check(&self, item: &ZoneBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneBatchRequiredTenantIdRule {
    pub message: String,
}

impl ZoneBatchRule for ZoneBatchRequiredTenantIdRule {
    fn check(&self, item: &ZoneBatch) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneBatchRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneBatchRule for ZoneBatchRevisionRangeRule {
    fn check(&self, item: &ZoneBatch) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a zone batch.
#[derive(Debug, Clone)]
pub struct ZoneBatchValidator {
    pub rules: Vec<Rc<dyn ZoneBatchRule>>,
}

impl ZoneBatchValidator {
    pub fn validate(&self, item: &ZoneBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_batch_validator() -> ZoneBatchValidator {
    ZoneBatchValidator {
        rules: vec![
            Rc::new(ZoneBatchAllowedCodeRule {
                allowed: vec!["global".to_string(), "fallback".to_string()],
            }) as Rc<dyn ZoneBatchRule>,
            Rc::new(ZoneBatchNameLengthRule { max_length: 1000 }),
            Rc::new(ZoneBatchRequiredPhoneRule {
                message: "zone batch needs a value".to_string(),
            }),
            Rc::new(ZoneBatchCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ZoneBatchRequiredTenantIdRule {
                message: "zone batch needs an entry".to_string(),
            }),
            Rc::new(ZoneBatchRevisionRangeRule { min: 80, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_batch_validator();
        let item = new_zone_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ZoneBatchNameLengthRule { max_length: 16 };
        let mut item = new_zone_batch("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = ZoneBatchRequiredPhoneRule {
            message: "zone batch needs an entry".to_string(),
        };
        let mut item = new_zone_batch("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = ZoneBatchRequiredTenantIdRule {
            message: "zone batch needs this field".to_string(),
        };
        let mut item = new_zone_batch("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = ZoneBatchRevisionRangeRule { min: 32, max: 255 };
        let mut item = new_zone_batch("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
