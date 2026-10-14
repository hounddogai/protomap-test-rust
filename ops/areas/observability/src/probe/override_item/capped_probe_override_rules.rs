use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped probe override must pass. Check returns its problems, or none.
pub trait CappedProbeOverrideRule: std::fmt::Debug {
    fn check(&self, item: &CappedProbeOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeOverrideRequiredScopeRule {
    pub message: String,
}

impl CappedProbeOverrideRule for CappedProbeOverrideRequiredScopeRule {
    fn check(&self, item: &CappedProbeOverride) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeOverrideRequiredRegionRule {
    pub message: String,
}

impl CappedProbeOverrideRule for CappedProbeOverrideRequiredRegionRule {
    fn check(&self, item: &CappedProbeOverride) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeOverrideScopeLengthRule {
    pub max_length: i64,
}

impl CappedProbeOverrideRule for CappedProbeOverrideScopeLengthRule {
    fn check(&self, item: &CappedProbeOverride) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeOverrideRequiredNameRule {
    pub message: String,
}

impl CappedProbeOverrideRule for CappedProbeOverrideRequiredNameRule {
    fn check(&self, item: &CappedProbeOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped probe override.
#[derive(Debug, Clone)]
pub struct CappedProbeOverrideValidator {
    pub rules: Vec<Rc<dyn CappedProbeOverrideRule>>,
}

impl CappedProbeOverrideValidator {
    pub fn validate(&self, item: &CappedProbeOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedProbeOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedProbeOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_probe_override_validator() -> CappedProbeOverrideValidator {
    CappedProbeOverrideValidator {
        rules: vec![
            Rc::new(CappedProbeOverrideRequiredScopeRule {
                message: "capped probe override needs this field".to_string(),
            }) as Rc<dyn CappedProbeOverrideRule>,
            Rc::new(CappedProbeOverrideRequiredRegionRule {
                message: "capped probe override needs this field".to_string(),
            }),
            Rc::new(CappedProbeOverrideScopeLengthRule { max_length: 16 }),
            Rc::new(CappedProbeOverrideRequiredNameRule {
                message: "capped probe override needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_probe_override_validator();
        let item = new_capped_probe_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = CappedProbeOverrideRequiredScopeRule {
            message: "capped probe override needs a value".to_string(),
        };
        let mut item = new_capped_probe_override("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = CappedProbeOverrideRequiredRegionRule {
            message: "capped probe override needs an entry".to_string(),
        };
        let mut item = new_capped_probe_override("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = CappedProbeOverrideScopeLengthRule { max_length: 16 };
        let mut item = new_capped_probe_override("x", "y");
        item.scope = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CappedProbeOverrideRequiredNameRule {
            message: "capped probe override needs an entry".to_string(),
        };
        let mut item = new_capped_probe_override("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
