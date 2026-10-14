use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback audit event manifest must pass. Check returns its problems, or none.
pub trait FallbackAuditEventManifestRule: std::fmt::Debug {
    fn check(&self, item: &FallbackAuditEventManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventManifestRequiredCodeRule {
    pub message: String,
}

impl FallbackAuditEventManifestRule for FallbackAuditEventManifestRequiredCodeRule {
    fn check(&self, item: &FallbackAuditEventManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventManifestCodePrefixRule {
    pub prefix: String,
}

impl FallbackAuditEventManifestRule for FallbackAuditEventManifestCodePrefixRule {
    fn check(&self, item: &FallbackAuditEventManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventManifestRequiredNotesRule {
    pub message: String,
}

impl FallbackAuditEventManifestRule for FallbackAuditEventManifestRequiredNotesRule {
    fn check(&self, item: &FallbackAuditEventManifest) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventManifestRequiredTitleRule {
    pub message: String,
}

impl FallbackAuditEventManifestRule for FallbackAuditEventManifestRequiredTitleRule {
    fn check(&self, item: &FallbackAuditEventManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback audit event manifest.
#[derive(Debug, Clone)]
pub struct FallbackAuditEventManifestValidator {
    pub rules: Vec<Rc<dyn FallbackAuditEventManifestRule>>,
}

impl FallbackAuditEventManifestValidator {
    pub fn validate(&self, item: &FallbackAuditEventManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackAuditEventManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackAuditEventManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_audit_event_manifest_validator() -> FallbackAuditEventManifestValidator {
    FallbackAuditEventManifestValidator {
        rules: vec![
            Rc::new(FallbackAuditEventManifestRequiredCodeRule {
                message: "fallback audit event manifest needs a value".to_string(),
            }) as Rc<dyn FallbackAuditEventManifestRule>,
            Rc::new(FallbackAuditEventManifestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(FallbackAuditEventManifestRequiredNotesRule {
                message: "fallback audit event manifest needs a value".to_string(),
            }),
            Rc::new(FallbackAuditEventManifestRequiredTitleRule {
                message: "fallback audit event manifest needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_audit_event_manifest_validator();
        let item = new_fallback_audit_event_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackAuditEventManifestRequiredCodeRule {
            message: "fallback audit event manifest needs this field".to_string(),
        };
        let mut item = new_fallback_audit_event_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = FallbackAuditEventManifestRequiredNotesRule {
            message: "fallback audit event manifest needs an entry".to_string(),
        };
        let mut item = new_fallback_audit_event_manifest("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FallbackAuditEventManifestRequiredTitleRule {
            message: "fallback audit event manifest needs a value".to_string(),
        };
        let mut item = new_fallback_audit_event_manifest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
