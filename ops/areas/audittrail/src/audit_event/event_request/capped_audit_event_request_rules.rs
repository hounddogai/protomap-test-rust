use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped audit event request must pass. Check returns its problems, or none.
pub trait CappedAuditEventRequestRule: std::fmt::Debug {
    fn check(&self, item: &CappedAuditEventRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAuditEventRequestQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedAuditEventRequestRule for CappedAuditEventRequestQuotaUnitsRangeRule {
    fn check(&self, item: &CappedAuditEventRequest) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAuditEventRequestRequiredCodeRule {
    pub message: String,
}

impl CappedAuditEventRequestRule for CappedAuditEventRequestRequiredCodeRule {
    fn check(&self, item: &CappedAuditEventRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAuditEventRequestQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedAuditEventRequestRule for CappedAuditEventRequestQuantityRangeRule {
    fn check(&self, item: &CappedAuditEventRequest) -> Vec<String> {
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
pub struct CappedAuditEventRequestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CappedAuditEventRequestRule for CappedAuditEventRequestAllowedCodeRule {
    fn check(&self, item: &CappedAuditEventRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped audit event request.
#[derive(Debug, Clone)]
pub struct CappedAuditEventRequestValidator {
    pub rules: Vec<Rc<dyn CappedAuditEventRequestRule>>,
}

impl CappedAuditEventRequestValidator {
    pub fn validate(&self, item: &CappedAuditEventRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedAuditEventRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedAuditEventRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_audit_event_request_validator() -> CappedAuditEventRequestValidator {
    CappedAuditEventRequestValidator {
        rules: vec![
            Rc::new(CappedAuditEventRequestQuotaUnitsRangeRule { min: 255, max: 3600 })
                as Rc<dyn CappedAuditEventRequestRule>,
            Rc::new(CappedAuditEventRequestRequiredCodeRule {
                message: "capped audit event request needs a value".to_string(),
            }),
            Rc::new(CappedAuditEventRequestQuantityRangeRule { min: 32, max: 64 }),
            Rc::new(CappedAuditEventRequestAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_audit_event_request_validator();
        let item = new_capped_audit_event_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = CappedAuditEventRequestQuotaUnitsRangeRule { min: 64, max: 1000 };
        let mut item = new_capped_audit_event_request("x", "y");
        item.quota_units = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedAuditEventRequestRequiredCodeRule {
            message: "capped audit event request needs an entry".to_string(),
        };
        let mut item = new_capped_audit_event_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = CappedAuditEventRequestQuantityRangeRule { min: 1000, max: 86400 };
        let mut item = new_capped_audit_event_request("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
