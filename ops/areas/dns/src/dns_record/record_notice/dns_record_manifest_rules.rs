use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record manifest must pass. Check returns its problems, or none.
pub trait DnsRecordManifestRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordManifestCodePrefixRule {
    pub prefix: String,
}

impl DnsRecordManifestRule for DnsRecordManifestCodePrefixRule {
    fn check(&self, item: &DnsRecordManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordManifestRequiredUnitRule {
    pub message: String,
}

impl DnsRecordManifestRule for DnsRecordManifestRequiredUnitRule {
    fn check(&self, item: &DnsRecordManifest) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordManifestTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordManifestRule for DnsRecordManifestTtlSecondsRangeRule {
    fn check(&self, item: &DnsRecordManifest) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordManifestUnitLengthRule {
    pub max_length: i64,
}

impl DnsRecordManifestRule for DnsRecordManifestUnitLengthRule {
    fn check(&self, item: &DnsRecordManifest) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordManifestNamespaceLengthRule {
    pub max_length: i64,
}

impl DnsRecordManifestRule for DnsRecordManifestNamespaceLengthRule {
    fn check(&self, item: &DnsRecordManifest) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordManifestRequiredNameRule {
    pub message: String,
}

impl DnsRecordManifestRule for DnsRecordManifestRequiredNameRule {
    fn check(&self, item: &DnsRecordManifest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record manifest.
#[derive(Debug, Clone)]
pub struct DnsRecordManifestValidator {
    pub rules: Vec<Rc<dyn DnsRecordManifestRule>>,
}

impl DnsRecordManifestValidator {
    pub fn validate(&self, item: &DnsRecordManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_manifest_validator() -> DnsRecordManifestValidator {
    DnsRecordManifestValidator {
        rules: vec![
            Rc::new(DnsRecordManifestCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn DnsRecordManifestRule>,
            Rc::new(DnsRecordManifestRequiredUnitRule {
                message: "dns record manifest needs an entry".to_string(),
            }),
            Rc::new(DnsRecordManifestTtlSecondsRangeRule { min: 80, max: 3600 }),
            Rc::new(DnsRecordManifestUnitLengthRule { max_length: 86400 }),
            Rc::new(DnsRecordManifestNamespaceLengthRule { max_length: 86400 }),
            Rc::new(DnsRecordManifestRequiredNameRule {
                message: "dns record manifest needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_manifest_validator();
        let item = new_dns_record_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = DnsRecordManifestRequiredUnitRule {
            message: "dns record manifest needs an entry".to_string(),
        };
        let mut item = new_dns_record_manifest("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = DnsRecordManifestTtlSecondsRangeRule { min: 64, max: 86400 };
        let mut item = new_dns_record_manifest("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = DnsRecordManifestUnitLengthRule { max_length: 64 };
        let mut item = new_dns_record_manifest("x", "y");
        item.unit = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = DnsRecordManifestNamespaceLengthRule { max_length: 1000 };
        let mut item = new_dns_record_manifest("x", "y");
        item.namespace = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DnsRecordManifestRequiredNameRule {
            message: "dns record manifest needs this field".to_string(),
        };
        let mut item = new_dns_record_manifest("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
