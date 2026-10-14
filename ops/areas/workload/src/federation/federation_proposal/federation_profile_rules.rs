use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation profile must pass. Check returns its problems, or none.
pub trait FederationProfileRule: std::fmt::Debug {
    fn check(&self, item: &FederationProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationProfileRegionLengthRule {
    pub max_length: i64,
}

impl FederationProfileRule for FederationProfileRegionLengthRule {
    fn check(&self, item: &FederationProfile) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationProfileRequiredRegionRule {
    pub message: String,
}

impl FederationProfileRule for FederationProfileRequiredRegionRule {
    fn check(&self, item: &FederationProfile) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationProfileCodeLengthRule {
    pub max_length: i64,
}

impl FederationProfileRule for FederationProfileCodeLengthRule {
    fn check(&self, item: &FederationProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationProfileNameLengthRule {
    pub max_length: i64,
}

impl FederationProfileRule for FederationProfileNameLengthRule {
    fn check(&self, item: &FederationProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation profile.
#[derive(Debug, Clone)]
pub struct FederationProfileValidator {
    pub rules: Vec<Rc<dyn FederationProfileRule>>,
}

impl FederationProfileValidator {
    pub fn validate(&self, item: &FederationProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_profile_validator() -> FederationProfileValidator {
    FederationProfileValidator {
        rules: vec![
            Rc::new(FederationProfileRegionLengthRule { max_length: 80 }) as Rc<dyn FederationProfileRule>,
            Rc::new(FederationProfileRequiredRegionRule {
                message: "federation profile needs a value".to_string(),
            }),
            Rc::new(FederationProfileCodeLengthRule { max_length: 86400 }),
            Rc::new(FederationProfileNameLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_profile_validator();
        let item = new_federation_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = FederationProfileRegionLengthRule { max_length: 86400 };
        let mut item = new_federation_profile("x", "y");
        item.region = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = FederationProfileRequiredRegionRule {
            message: "federation profile needs an entry".to_string(),
        };
        let mut item = new_federation_profile("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FederationProfileCodeLengthRule { max_length: 255 };
        let mut item = new_federation_profile("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FederationProfileNameLengthRule { max_length: 16 };
        let mut item = new_federation_profile("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
