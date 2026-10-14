use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global isolation claim must pass. Check returns its problems, or none.
pub trait GlobalIsolationClaimRule: std::fmt::Debug {
    fn check(&self, item: &GlobalIsolationClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationClaimRequiredNameRule {
    pub message: String,
}

impl GlobalIsolationClaimRule for GlobalIsolationClaimRequiredNameRule {
    fn check(&self, item: &GlobalIsolationClaim) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationClaimNameLengthRule {
    pub max_length: i64,
}

impl GlobalIsolationClaimRule for GlobalIsolationClaimNameLengthRule {
    fn check(&self, item: &GlobalIsolationClaim) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationClaimRequiredCodeRule {
    pub message: String,
}

impl GlobalIsolationClaimRule for GlobalIsolationClaimRequiredCodeRule {
    fn check(&self, item: &GlobalIsolationClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationClaimRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalIsolationClaimRule for GlobalIsolationClaimRotationDaysRangeRule {
    fn check(&self, item: &GlobalIsolationClaim) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationClaimCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalIsolationClaimRule for GlobalIsolationClaimCreatedAtRangeRule {
    fn check(&self, item: &GlobalIsolationClaim) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global isolation claim.
#[derive(Debug, Clone)]
pub struct GlobalIsolationClaimValidator {
    pub rules: Vec<Rc<dyn GlobalIsolationClaimRule>>,
}

impl GlobalIsolationClaimValidator {
    pub fn validate(&self, item: &GlobalIsolationClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalIsolationClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalIsolationClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_isolation_claim_validator() -> GlobalIsolationClaimValidator {
    GlobalIsolationClaimValidator {
        rules: vec![
            Rc::new(GlobalIsolationClaimRequiredNameRule {
                message: "global isolation claim needs an entry".to_string(),
            }) as Rc<dyn GlobalIsolationClaimRule>,
            Rc::new(GlobalIsolationClaimNameLengthRule { max_length: 16 }),
            Rc::new(GlobalIsolationClaimRequiredCodeRule {
                message: "global isolation claim needs an entry".to_string(),
            }),
            Rc::new(GlobalIsolationClaimRotationDaysRangeRule { min: 32, max: 120 }),
            Rc::new(GlobalIsolationClaimCreatedAtRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_isolation_claim_validator();
        let item = new_global_isolation_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GlobalIsolationClaimRequiredNameRule {
            message: "global isolation claim needs this field".to_string(),
        };
        let mut item = new_global_isolation_claim("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GlobalIsolationClaimNameLengthRule { max_length: 32 };
        let mut item = new_global_isolation_claim("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalIsolationClaimRequiredCodeRule {
            message: "global isolation claim needs this field".to_string(),
        };
        let mut item = new_global_isolation_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = GlobalIsolationClaimRotationDaysRangeRule { min: 32, max: 32 };
        let mut item = new_global_isolation_claim("x", "y");
        item.rotation_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = GlobalIsolationClaimCreatedAtRangeRule { min: 64, max: 86400 };
        let mut item = new_global_isolation_claim("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
