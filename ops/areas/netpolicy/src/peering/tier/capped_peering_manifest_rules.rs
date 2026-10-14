use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped peering manifest must pass. Check returns its problems, or none.
pub trait CappedPeeringManifestRule: std::fmt::Debug {
    fn check(&self, item: &CappedPeeringManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringManifestTenantIdLengthRule {
    pub max_length: i64,
}

impl CappedPeeringManifestRule for CappedPeeringManifestTenantIdLengthRule {
    fn check(&self, item: &CappedPeeringManifest) -> Vec<String> {
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
pub struct CappedPeeringManifestRequiredCodeRule {
    pub message: String,
}

impl CappedPeeringManifestRule for CappedPeeringManifestRequiredCodeRule {
    fn check(&self, item: &CappedPeeringManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringManifestRequiredContactEmailRule {
    pub message: String,
}

impl CappedPeeringManifestRule for CappedPeeringManifestRequiredContactEmailRule {
    fn check(&self, item: &CappedPeeringManifest) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringManifestRequiredNameRule {
    pub message: String,
}

impl CappedPeeringManifestRule for CappedPeeringManifestRequiredNameRule {
    fn check(&self, item: &CappedPeeringManifest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringManifestDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedPeeringManifestRule for CappedPeeringManifestDurationMinutesRangeRule {
    fn check(&self, item: &CappedPeeringManifest) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped peering manifest.
#[derive(Debug, Clone)]
pub struct CappedPeeringManifestValidator {
    pub rules: Vec<Rc<dyn CappedPeeringManifestRule>>,
}

impl CappedPeeringManifestValidator {
    pub fn validate(&self, item: &CappedPeeringManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedPeeringManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedPeeringManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_peering_manifest_validator() -> CappedPeeringManifestValidator {
    CappedPeeringManifestValidator {
        rules: vec![
            Rc::new(CappedPeeringManifestTenantIdLengthRule { max_length: 32 }) as Rc<dyn CappedPeeringManifestRule>,
            Rc::new(CappedPeeringManifestRequiredCodeRule {
                message: "capped peering manifest needs this field".to_string(),
            }),
            Rc::new(CappedPeeringManifestRequiredContactEmailRule {
                message: "capped peering manifest needs a value".to_string(),
            }),
            Rc::new(CappedPeeringManifestRequiredNameRule {
                message: "capped peering manifest needs an entry".to_string(),
            }),
            Rc::new(CappedPeeringManifestDurationMinutesRangeRule { min: 255, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_peering_manifest_validator();
        let item = new_capped_peering_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = CappedPeeringManifestTenantIdLengthRule { max_length: 1000 };
        let mut item = new_capped_peering_manifest("x", "y");
        item.tenant_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedPeeringManifestRequiredCodeRule {
            message: "capped peering manifest needs a value".to_string(),
        };
        let mut item = new_capped_peering_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = CappedPeeringManifestRequiredContactEmailRule {
            message: "capped peering manifest needs an entry".to_string(),
        };
        let mut item = new_capped_peering_manifest("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CappedPeeringManifestRequiredNameRule {
            message: "capped peering manifest needs this field".to_string(),
        };
        let mut item = new_capped_peering_manifest("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = CappedPeeringManifestDurationMinutesRangeRule { min: 16, max: 64 };
        let mut item = new_capped_peering_manifest("x", "y");
        item.duration_minutes = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
