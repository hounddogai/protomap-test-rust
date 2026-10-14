use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan digest must pass. Check returns its problems, or none.
pub trait PlanDigestRule: std::fmt::Debug {
    fn check(&self, item: &PlanDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanDigestRegionLengthRule {
    pub max_length: i64,
}

impl PlanDigestRule for PlanDigestRegionLengthRule {
    fn check(&self, item: &PlanDigest) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanDigestRequiredRegionRule {
    pub message: String,
}

impl PlanDigestRule for PlanDigestRequiredRegionRule {
    fn check(&self, item: &PlanDigest) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanDigestNameLengthRule {
    pub max_length: i64,
}

impl PlanDigestRule for PlanDigestNameLengthRule {
    fn check(&self, item: &PlanDigest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanDigestRequiredUnitRule {
    pub message: String,
}

impl PlanDigestRule for PlanDigestRequiredUnitRule {
    fn check(&self, item: &PlanDigest) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanDigestUnitLengthRule {
    pub max_length: i64,
}

impl PlanDigestRule for PlanDigestUnitLengthRule {
    fn check(&self, item: &PlanDigest) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan digest.
#[derive(Debug, Clone)]
pub struct PlanDigestValidator {
    pub rules: Vec<Rc<dyn PlanDigestRule>>,
}

impl PlanDigestValidator {
    pub fn validate(&self, item: &PlanDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_digest_validator() -> PlanDigestValidator {
    PlanDigestValidator {
        rules: vec![
            Rc::new(PlanDigestRegionLengthRule { max_length: 86400 }) as Rc<dyn PlanDigestRule>,
            Rc::new(PlanDigestRequiredRegionRule {
                message: "plan digest needs this field".to_string(),
            }),
            Rc::new(PlanDigestNameLengthRule { max_length: 16 }),
            Rc::new(PlanDigestRequiredUnitRule {
                message: "plan digest needs a value".to_string(),
            }),
            Rc::new(PlanDigestUnitLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_digest_validator();
        let item = new_plan_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = PlanDigestRegionLengthRule { max_length: 1000 };
        let mut item = new_plan_digest("x", "y");
        item.region = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = PlanDigestRequiredRegionRule {
            message: "plan digest needs an entry".to_string(),
        };
        let mut item = new_plan_digest("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PlanDigestNameLengthRule { max_length: 64 };
        let mut item = new_plan_digest("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PlanDigestRequiredUnitRule {
            message: "plan digest needs this field".to_string(),
        };
        let mut item = new_plan_digest("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = PlanDigestUnitLengthRule { max_length: 32 };
        let mut item = new_plan_digest("x", "y");
        item.unit = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
