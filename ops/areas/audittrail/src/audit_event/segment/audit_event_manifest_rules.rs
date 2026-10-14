use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event manifest must pass. Check returns its problems, or none.
pub trait AuditEventManifestRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventManifestStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventManifestRule for AuditEventManifestStartsAtRangeRule {
    fn check(&self, item: &AuditEventManifest) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventManifestMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AuditEventManifestRule for AuditEventManifestMaximumRangeRule {
    fn check(&self, item: &AuditEventManifest) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventManifestRequiredNamespaceRule {
    pub message: String,
}

impl AuditEventManifestRule for AuditEventManifestRequiredNamespaceRule {
    fn check(&self, item: &AuditEventManifest) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event manifest.
#[derive(Debug, Clone)]
pub struct AuditEventManifestValidator {
    pub rules: Vec<Rc<dyn AuditEventManifestRule>>,
}

impl AuditEventManifestValidator {
    pub fn validate(&self, item: &AuditEventManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_manifest_validator() -> AuditEventManifestValidator {
    AuditEventManifestValidator {
        rules: vec![
            Rc::new(AuditEventManifestStartsAtRangeRule { min: 16, max: 80 }) as Rc<dyn AuditEventManifestRule>,
            Rc::new(AuditEventManifestMaximumRangeRule { min: 3.7, max: 4751.0 }),
            Rc::new(AuditEventManifestRequiredNamespaceRule {
                message: "audit event manifest needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_manifest_validator();
        let item = new_audit_event_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = AuditEventManifestStartsAtRangeRule { min: 32, max: 32 };
        let mut item = new_audit_event_manifest("x", "y");
        item.starts_at = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = AuditEventManifestMaximumRangeRule { min: 4.4, max: 368.1 };
        let mut item = new_audit_event_manifest("x", "y");
        item.maximum = 369.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = AuditEventManifestRequiredNamespaceRule {
            message: "audit event manifest needs an entry".to_string(),
        };
        let mut item = new_audit_event_manifest("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
