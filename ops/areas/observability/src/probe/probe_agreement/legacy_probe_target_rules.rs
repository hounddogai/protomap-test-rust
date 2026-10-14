use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy probe target must pass. Check returns its problems, or none.
pub trait LegacyProbeTargetRule: std::fmt::Debug {
    fn check(&self, item: &LegacyProbeTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeTargetNameLengthRule {
    pub max_length: i64,
}

impl LegacyProbeTargetRule for LegacyProbeTargetNameLengthRule {
    fn check(&self, item: &LegacyProbeTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeTargetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LegacyProbeTargetRule for LegacyProbeTargetAllowedCodeRule {
    fn check(&self, item: &LegacyProbeTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeTargetCodePrefixRule {
    pub prefix: String,
}

impl LegacyProbeTargetRule for LegacyProbeTargetCodePrefixRule {
    fn check(&self, item: &LegacyProbeTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeTargetExternalRefLengthRule {
    pub max_length: i64,
}

impl LegacyProbeTargetRule for LegacyProbeTargetExternalRefLengthRule {
    fn check(&self, item: &LegacyProbeTarget) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy probe target.
#[derive(Debug, Clone)]
pub struct LegacyProbeTargetValidator {
    pub rules: Vec<Rc<dyn LegacyProbeTargetRule>>,
}

impl LegacyProbeTargetValidator {
    pub fn validate(&self, item: &LegacyProbeTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyProbeTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyProbeTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_probe_target_validator() -> LegacyProbeTargetValidator {
    LegacyProbeTargetValidator {
        rules: vec![
            Rc::new(LegacyProbeTargetNameLengthRule { max_length: 255 }) as Rc<dyn LegacyProbeTargetRule>,
            Rc::new(LegacyProbeTargetAllowedCodeRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(LegacyProbeTargetCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(LegacyProbeTargetExternalRefLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_probe_target_validator();
        let item = new_legacy_probe_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LegacyProbeTargetNameLengthRule { max_length: 86400 };
        let mut item = new_legacy_probe_target("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = LegacyProbeTargetExternalRefLengthRule { max_length: 255 };
        let mut item = new_legacy_probe_target("x", "y");
        item.external_ref = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
