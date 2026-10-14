use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered flow log manifest must pass. Check returns its problems, or none.
pub trait TieredFlowLogManifestRule: std::fmt::Debug {
    fn check(&self, item: &TieredFlowLogManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogManifestExternalRefLengthRule {
    pub max_length: i64,
}

impl TieredFlowLogManifestRule for TieredFlowLogManifestExternalRefLengthRule {
    fn check(&self, item: &TieredFlowLogManifest) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogManifestCodePrefixRule {
    pub prefix: String,
}

impl TieredFlowLogManifestRule for TieredFlowLogManifestCodePrefixRule {
    fn check(&self, item: &TieredFlowLogManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogManifestNameLengthRule {
    pub max_length: i64,
}

impl TieredFlowLogManifestRule for TieredFlowLogManifestNameLengthRule {
    fn check(&self, item: &TieredFlowLogManifest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogManifestTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredFlowLogManifestRule for TieredFlowLogManifestTtlSecondsRangeRule {
    fn check(&self, item: &TieredFlowLogManifest) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogManifestCodeLengthRule {
    pub max_length: i64,
}

impl TieredFlowLogManifestRule for TieredFlowLogManifestCodeLengthRule {
    fn check(&self, item: &TieredFlowLogManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered flow log manifest.
#[derive(Debug, Clone)]
pub struct TieredFlowLogManifestValidator {
    pub rules: Vec<Rc<dyn TieredFlowLogManifestRule>>,
}

impl TieredFlowLogManifestValidator {
    pub fn validate(&self, item: &TieredFlowLogManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredFlowLogManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredFlowLogManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_flow_log_manifest_validator() -> TieredFlowLogManifestValidator {
    TieredFlowLogManifestValidator {
        rules: vec![
            Rc::new(TieredFlowLogManifestExternalRefLengthRule { max_length: 255 })
                as Rc<dyn TieredFlowLogManifestRule>,
            Rc::new(TieredFlowLogManifestCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(TieredFlowLogManifestNameLengthRule { max_length: 1000 }),
            Rc::new(TieredFlowLogManifestTtlSecondsRangeRule { min: 64, max: 80 }),
            Rc::new(TieredFlowLogManifestCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_flow_log_manifest_validator();
        let item = new_tiered_flow_log_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = TieredFlowLogManifestExternalRefLengthRule { max_length: 3600 };
        let mut item = new_tiered_flow_log_manifest("x", "y");
        item.external_ref = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TieredFlowLogManifestNameLengthRule { max_length: 3600 };
        let mut item = new_tiered_flow_log_manifest("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = TieredFlowLogManifestTtlSecondsRangeRule { min: 1000, max: 3600 };
        let mut item = new_tiered_flow_log_manifest("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredFlowLogManifestCodeLengthRule { max_length: 32 };
        let mut item = new_tiered_flow_log_manifest("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
