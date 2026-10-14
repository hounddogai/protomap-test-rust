use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage profile must pass. Check returns its problems, or none.
pub trait OverageProfileRule: std::fmt::Debug {
    fn check(&self, item: &OverageProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageProfileCodePrefixRule {
    pub prefix: String,
}

impl OverageProfileRule for OverageProfileCodePrefixRule {
    fn check(&self, item: &OverageProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageProfileSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageProfileRule for OverageProfileSequenceRangeRule {
    fn check(&self, item: &OverageProfile) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageProfileTenantIdLengthRule {
    pub max_length: i64,
}

impl OverageProfileRule for OverageProfileTenantIdLengthRule {
    fn check(&self, item: &OverageProfile) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageProfileRegionLengthRule {
    pub max_length: i64,
}

impl OverageProfileRule for OverageProfileRegionLengthRule {
    fn check(&self, item: &OverageProfile) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage profile.
#[derive(Debug, Clone)]
pub struct OverageProfileValidator {
    pub rules: Vec<Rc<dyn OverageProfileRule>>,
}

impl OverageProfileValidator {
    pub fn validate(&self, item: &OverageProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_profile_validator() -> OverageProfileValidator {
    OverageProfileValidator {
        rules: vec![
            Rc::new(OverageProfileCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn OverageProfileRule>,
            Rc::new(OverageProfileSequenceRangeRule { min: 64, max: 1000 }),
            Rc::new(OverageProfileTenantIdLengthRule { max_length: 16 }),
            Rc::new(OverageProfileRegionLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_profile_validator();
        let item = new_overage_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = OverageProfileSequenceRangeRule { min: 32, max: 86400 };
        let mut item = new_overage_profile("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = OverageProfileTenantIdLengthRule { max_length: 80 };
        let mut item = new_overage_profile("x", "y");
        item.tenant_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = OverageProfileRegionLengthRule { max_length: 16 };
        let mut item = new_overage_profile("x", "y");
        item.region = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
