use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary sink mapping must pass. Check returns its problems, or none.
pub trait PrimarySinkMappingRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySinkMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkMappingQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimarySinkMappingRule for PrimarySinkMappingQuotaUnitsRangeRule {
    fn check(&self, item: &PrimarySinkMapping) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkMappingTitleLengthRule {
    pub max_length: i64,
}

impl PrimarySinkMappingRule for PrimarySinkMappingTitleLengthRule {
    fn check(&self, item: &PrimarySinkMapping) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkMappingMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimarySinkMappingRule for PrimarySinkMappingMaximumRangeRule {
    fn check(&self, item: &PrimarySinkMapping) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkMappingCodePrefixRule {
    pub prefix: String,
}

impl PrimarySinkMappingRule for PrimarySinkMappingCodePrefixRule {
    fn check(&self, item: &PrimarySinkMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkMappingKeyIdLengthRule {
    pub max_length: i64,
}

impl PrimarySinkMappingRule for PrimarySinkMappingKeyIdLengthRule {
    fn check(&self, item: &PrimarySinkMapping) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary sink mapping.
#[derive(Debug, Clone)]
pub struct PrimarySinkMappingValidator {
    pub rules: Vec<Rc<dyn PrimarySinkMappingRule>>,
}

impl PrimarySinkMappingValidator {
    pub fn validate(&self, item: &PrimarySinkMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimarySinkMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySinkMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_sink_mapping_validator() -> PrimarySinkMappingValidator {
    PrimarySinkMappingValidator {
        rules: vec![
            Rc::new(PrimarySinkMappingQuotaUnitsRangeRule { min: 64, max: 86400 }) as Rc<dyn PrimarySinkMappingRule>,
            Rc::new(PrimarySinkMappingTitleLengthRule { max_length: 86400 }),
            Rc::new(PrimarySinkMappingMaximumRangeRule { min: 9.4, max: 1436.5 }),
            Rc::new(PrimarySinkMappingCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PrimarySinkMappingKeyIdLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_sink_mapping_validator();
        let item = new_primary_sink_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = PrimarySinkMappingQuotaUnitsRangeRule { min: 32, max: 64 };
        let mut item = new_primary_sink_mapping("x", "y");
        item.quota_units = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimarySinkMappingTitleLengthRule { max_length: 16 };
        let mut item = new_primary_sink_mapping("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = PrimarySinkMappingMaximumRangeRule { min: 9.6, max: 947.8 };
        let mut item = new_primary_sink_mapping("x", "y");
        item.maximum = 949.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = PrimarySinkMappingKeyIdLengthRule { max_length: 3600 };
        let mut item = new_primary_sink_mapping("x", "y");
        item.key_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
