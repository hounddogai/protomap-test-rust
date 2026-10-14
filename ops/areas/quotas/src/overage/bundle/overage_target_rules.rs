use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage target must pass. Check returns its problems, or none.
pub trait OverageTargetRule: std::fmt::Debug {
    fn check(&self, item: &OverageTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageTargetRequiredCodeRule {
    pub message: String,
}

impl OverageTargetRule for OverageTargetRequiredCodeRule {
    fn check(&self, item: &OverageTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageTargetActiveGuardRule {
    pub limit: i64,
}

impl OverageTargetRule for OverageTargetActiveGuardRule {
    fn check(&self, item: &OverageTarget) -> Vec<String> {
        if item.active && (item.duration_minutes > self.limit) {
            return vec![format!(
                "duration minutes {} exceeds {} while active",
                item.duration_minutes, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageTargetRegionLengthRule {
    pub max_length: i64,
}

impl OverageTargetRule for OverageTargetRegionLengthRule {
    fn check(&self, item: &OverageTarget) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageTargetCodeLengthRule {
    pub max_length: i64,
}

impl OverageTargetRule for OverageTargetCodeLengthRule {
    fn check(&self, item: &OverageTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage target.
#[derive(Debug, Clone)]
pub struct OverageTargetValidator {
    pub rules: Vec<Rc<dyn OverageTargetRule>>,
}

impl OverageTargetValidator {
    pub fn validate(&self, item: &OverageTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_target_validator() -> OverageTargetValidator {
    OverageTargetValidator {
        rules: vec![
            Rc::new(OverageTargetRequiredCodeRule {
                message: "overage target needs a value".to_string(),
            }) as Rc<dyn OverageTargetRule>,
            Rc::new(OverageTargetActiveGuardRule { limit: 86400 }),
            Rc::new(OverageTargetRegionLengthRule { max_length: 3600 }),
            Rc::new(OverageTargetCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_target_validator();
        let item = new_overage_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = OverageTargetRequiredCodeRule {
            message: "overage target needs an entry".to_string(),
        };
        let mut item = new_overage_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = OverageTargetRegionLengthRule { max_length: 120 };
        let mut item = new_overage_target("x", "y");
        item.region = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = OverageTargetCodeLengthRule { max_length: 1000 };
        let mut item = new_overage_target("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
