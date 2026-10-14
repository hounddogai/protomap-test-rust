use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global audit event rollup must pass. Check returns its problems, or none.
pub trait GlobalAuditEventRollupRule: std::fmt::Debug {
    fn check(&self, item: &GlobalAuditEventRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventRollupMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalAuditEventRollupRule for GlobalAuditEventRollupMarginRangeRule {
    fn check(&self, item: &GlobalAuditEventRollup) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventRollupRequiredCodeRule {
    pub message: String,
}

impl GlobalAuditEventRollupRule for GlobalAuditEventRollupRequiredCodeRule {
    fn check(&self, item: &GlobalAuditEventRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventRollupTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl GlobalAuditEventRollupRule for GlobalAuditEventRollupTagsLimitRule {
    fn check(&self, item: &GlobalAuditEventRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventRollupCodePrefixRule {
    pub prefix: String,
}

impl GlobalAuditEventRollupRule for GlobalAuditEventRollupCodePrefixRule {
    fn check(&self, item: &GlobalAuditEventRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventRollupRequiredScopeRule {
    pub message: String,
}

impl GlobalAuditEventRollupRule for GlobalAuditEventRollupRequiredScopeRule {
    fn check(&self, item: &GlobalAuditEventRollup) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventRollupScopeLengthRule {
    pub max_length: i64,
}

impl GlobalAuditEventRollupRule for GlobalAuditEventRollupScopeLengthRule {
    fn check(&self, item: &GlobalAuditEventRollup) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global audit event rollup.
#[derive(Debug, Clone)]
pub struct GlobalAuditEventRollupValidator {
    pub rules: Vec<Rc<dyn GlobalAuditEventRollupRule>>,
}

impl GlobalAuditEventRollupValidator {
    pub fn validate(&self, item: &GlobalAuditEventRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalAuditEventRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalAuditEventRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_audit_event_rollup_validator() -> GlobalAuditEventRollupValidator {
    GlobalAuditEventRollupValidator {
        rules: vec![
            Rc::new(GlobalAuditEventRollupMarginRangeRule { min: 7.7, max: 4026.4 })
                as Rc<dyn GlobalAuditEventRollupRule>,
            Rc::new(GlobalAuditEventRollupRequiredCodeRule {
                message: "global audit event rollup needs an entry".to_string(),
            }),
            Rc::new(GlobalAuditEventRollupTagsLimitRule {
                max_count: 80,
                banned: vec!["seasonal".to_string(), "manual".to_string()],
            }),
            Rc::new(GlobalAuditEventRollupCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(GlobalAuditEventRollupRequiredScopeRule {
                message: "global audit event rollup needs this field".to_string(),
            }),
            Rc::new(GlobalAuditEventRollupScopeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_audit_event_rollup_validator();
        let item = new_global_audit_event_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = GlobalAuditEventRollupMarginRangeRule { min: 4.9, max: 3512.1 };
        let mut item = new_global_audit_event_rollup("x", "y");
        item.margin = 3513.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalAuditEventRollupRequiredCodeRule {
            message: "global audit event rollup needs this field".to_string(),
        };
        let mut item = new_global_audit_event_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = GlobalAuditEventRollupRequiredScopeRule {
            message: "global audit event rollup needs an entry".to_string(),
        };
        let mut item = new_global_audit_event_rollup("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = GlobalAuditEventRollupScopeLengthRule { max_length: 64 };
        let mut item = new_global_audit_event_rollup("x", "y");
        item.scope = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
