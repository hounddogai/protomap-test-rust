use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive audit event entry must pass. Check returns its problems, or none.
pub trait AdaptiveAuditEventEntryRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveAuditEventEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventEntryRequiredSegmentRule {
    pub message: String,
}

impl AdaptiveAuditEventEntryRule for AdaptiveAuditEventEntryRequiredSegmentRule {
    fn check(&self, item: &AdaptiveAuditEventEntry) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventEntryRequiredCodeRule {
    pub message: String,
}

impl AdaptiveAuditEventEntryRule for AdaptiveAuditEventEntryRequiredCodeRule {
    fn check(&self, item: &AdaptiveAuditEventEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventEntryPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveAuditEventEntryRule for AdaptiveAuditEventEntryPortRangeRule {
    fn check(&self, item: &AdaptiveAuditEventEntry) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventEntryCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveAuditEventEntryRule for AdaptiveAuditEventEntryCodeLengthRule {
    fn check(&self, item: &AdaptiveAuditEventEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive audit event entry.
#[derive(Debug, Clone)]
pub struct AdaptiveAuditEventEntryValidator {
    pub rules: Vec<Rc<dyn AdaptiveAuditEventEntryRule>>,
}

impl AdaptiveAuditEventEntryValidator {
    pub fn validate(&self, item: &AdaptiveAuditEventEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveAuditEventEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveAuditEventEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_audit_event_entry_validator() -> AdaptiveAuditEventEntryValidator {
    AdaptiveAuditEventEntryValidator {
        rules: vec![
            Rc::new(AdaptiveAuditEventEntryRequiredSegmentRule {
                message: "adaptive audit event entry needs an entry".to_string(),
            }) as Rc<dyn AdaptiveAuditEventEntryRule>,
            Rc::new(AdaptiveAuditEventEntryRequiredCodeRule {
                message: "adaptive audit event entry needs a value".to_string(),
            }),
            Rc::new(AdaptiveAuditEventEntryPortRangeRule { min: 16, max: 255 }),
            Rc::new(AdaptiveAuditEventEntryCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_audit_event_entry_validator();
        let item = new_adaptive_audit_event_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = AdaptiveAuditEventEntryRequiredSegmentRule {
            message: "adaptive audit event entry needs an entry".to_string(),
        };
        let mut item = new_adaptive_audit_event_entry("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveAuditEventEntryRequiredCodeRule {
            message: "adaptive audit event entry needs this field".to_string(),
        };
        let mut item = new_adaptive_audit_event_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = AdaptiveAuditEventEntryPortRangeRule { min: 16, max: 32 };
        let mut item = new_adaptive_audit_event_entry("x", "y");
        item.port = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveAuditEventEntryCodeLengthRule { max_length: 86400 };
        let mut item = new_adaptive_audit_event_entry("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
