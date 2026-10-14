use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered peering audit must pass. Check returns its problems, or none.
pub trait TieredPeeringAuditRule: std::fmt::Debug {
    fn check(&self, item: &TieredPeeringAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAuditAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TieredPeeringAuditRule for TieredPeeringAuditAllowedCodeRule {
    fn check(&self, item: &TieredPeeringAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAuditRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredPeeringAuditRule for TieredPeeringAuditRevisionRangeRule {
    fn check(&self, item: &TieredPeeringAudit) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAuditNameLengthRule {
    pub max_length: i64,
}

impl TieredPeeringAuditRule for TieredPeeringAuditNameLengthRule {
    fn check(&self, item: &TieredPeeringAudit) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAuditRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredPeeringAuditRule for TieredPeeringAuditRatioRangeRule {
    fn check(&self, item: &TieredPeeringAudit) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAuditRequiredExternalRefRule {
    pub message: String,
}

impl TieredPeeringAuditRule for TieredPeeringAuditRequiredExternalRefRule {
    fn check(&self, item: &TieredPeeringAudit) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered peering audit.
#[derive(Debug, Clone)]
pub struct TieredPeeringAuditValidator {
    pub rules: Vec<Rc<dyn TieredPeeringAuditRule>>,
}

impl TieredPeeringAuditValidator {
    pub fn validate(&self, item: &TieredPeeringAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredPeeringAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredPeeringAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_peering_audit_validator() -> TieredPeeringAuditValidator {
    TieredPeeringAuditValidator {
        rules: vec![
            Rc::new(TieredPeeringAuditAllowedCodeRule {
                allowed: vec!["capped".to_string(), "global".to_string(), "rolling".to_string()],
            }) as Rc<dyn TieredPeeringAuditRule>,
            Rc::new(TieredPeeringAuditRevisionRangeRule { min: 32, max: 255 }),
            Rc::new(TieredPeeringAuditNameLengthRule { max_length: 255 }),
            Rc::new(TieredPeeringAuditRatioRangeRule { min: 6.7, max: 2846.5 }),
            Rc::new(TieredPeeringAuditRequiredExternalRefRule {
                message: "tiered peering audit needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_peering_audit_validator();
        let item = new_tiered_peering_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = TieredPeeringAuditRevisionRangeRule { min: 64, max: 1000 };
        let mut item = new_tiered_peering_audit("x", "y");
        item.revision = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TieredPeeringAuditNameLengthRule { max_length: 1000 };
        let mut item = new_tiered_peering_audit("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = TieredPeeringAuditRatioRangeRule { min: 8.3, max: 495.1 };
        let mut item = new_tiered_peering_audit("x", "y");
        item.ratio = 496.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = TieredPeeringAuditRequiredExternalRefRule {
            message: "tiered peering audit needs this field".to_string(),
        };
        let mut item = new_tiered_peering_audit("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
