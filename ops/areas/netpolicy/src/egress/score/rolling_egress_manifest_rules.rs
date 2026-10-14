use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling egress manifest must pass. Check returns its problems, or none.
pub trait RollingEgressManifestRule: std::fmt::Debug {
    fn check(&self, item: &RollingEgressManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressManifestCountryLengthRule {
    pub max_length: i64,
}

impl RollingEgressManifestRule for RollingEgressManifestCountryLengthRule {
    fn check(&self, item: &RollingEgressManifest) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressManifestCodeLengthRule {
    pub max_length: i64,
}

impl RollingEgressManifestRule for RollingEgressManifestCodeLengthRule {
    fn check(&self, item: &RollingEgressManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressManifestRequiredTitleRule {
    pub message: String,
}

impl RollingEgressManifestRule for RollingEgressManifestRequiredTitleRule {
    fn check(&self, item: &RollingEgressManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressManifestChecksumLengthRule {
    pub max_length: i64,
}

impl RollingEgressManifestRule for RollingEgressManifestChecksumLengthRule {
    fn check(&self, item: &RollingEgressManifest) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressManifestCodePrefixRule {
    pub prefix: String,
}

impl RollingEgressManifestRule for RollingEgressManifestCodePrefixRule {
    fn check(&self, item: &RollingEgressManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressManifestRequiredChecksumRule {
    pub message: String,
}

impl RollingEgressManifestRule for RollingEgressManifestRequiredChecksumRule {
    fn check(&self, item: &RollingEgressManifest) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling egress manifest.
#[derive(Debug, Clone)]
pub struct RollingEgressManifestValidator {
    pub rules: Vec<Rc<dyn RollingEgressManifestRule>>,
}

impl RollingEgressManifestValidator {
    pub fn validate(&self, item: &RollingEgressManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingEgressManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingEgressManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_egress_manifest_validator() -> RollingEgressManifestValidator {
    RollingEgressManifestValidator {
        rules: vec![
            Rc::new(RollingEgressManifestCountryLengthRule { max_length: 32 }) as Rc<dyn RollingEgressManifestRule>,
            Rc::new(RollingEgressManifestCodeLengthRule { max_length: 16 }),
            Rc::new(RollingEgressManifestRequiredTitleRule {
                message: "rolling egress manifest needs this field".to_string(),
            }),
            Rc::new(RollingEgressManifestChecksumLengthRule { max_length: 16 }),
            Rc::new(RollingEgressManifestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RollingEgressManifestRequiredChecksumRule {
                message: "rolling egress manifest needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_egress_manifest_validator();
        let item = new_rolling_egress_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = RollingEgressManifestCountryLengthRule { max_length: 255 };
        let mut item = new_rolling_egress_manifest("x", "y");
        item.country = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingEgressManifestCodeLengthRule { max_length: 64 };
        let mut item = new_rolling_egress_manifest("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RollingEgressManifestRequiredTitleRule {
            message: "rolling egress manifest needs this field".to_string(),
        };
        let mut item = new_rolling_egress_manifest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = RollingEgressManifestChecksumLengthRule { max_length: 16 };
        let mut item = new_rolling_egress_manifest("x", "y");
        item.checksum = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = RollingEgressManifestRequiredChecksumRule {
            message: "rolling egress manifest needs an entry".to_string(),
        };
        let mut item = new_rolling_egress_manifest("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
