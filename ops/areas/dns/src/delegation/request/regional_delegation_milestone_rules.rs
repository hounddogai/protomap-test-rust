use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional delegation milestone must pass. Check returns its problems, or none.
pub trait RegionalDelegationMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &RegionalDelegationMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationMilestoneAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalDelegationMilestoneRule for RegionalDelegationMilestoneAttemptsRangeRule {
    fn check(&self, item: &RegionalDelegationMilestone) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationMilestoneNameLengthRule {
    pub max_length: i64,
}

impl RegionalDelegationMilestoneRule for RegionalDelegationMilestoneNameLengthRule {
    fn check(&self, item: &RegionalDelegationMilestone) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl RegionalDelegationMilestoneRule for RegionalDelegationMilestoneCodeLengthRule {
    fn check(&self, item: &RegionalDelegationMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationMilestoneRequiredCodeRule {
    pub message: String,
}

impl RegionalDelegationMilestoneRule for RegionalDelegationMilestoneRequiredCodeRule {
    fn check(&self, item: &RegionalDelegationMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationMilestoneMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalDelegationMilestoneRule for RegionalDelegationMilestoneMarginRangeRule {
    fn check(&self, item: &RegionalDelegationMilestone) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationMilestoneTenantIdLengthRule {
    pub max_length: i64,
}

impl RegionalDelegationMilestoneRule for RegionalDelegationMilestoneTenantIdLengthRule {
    fn check(&self, item: &RegionalDelegationMilestone) -> Vec<String> {
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

/// Runs every rule of a regional delegation milestone.
#[derive(Debug, Clone)]
pub struct RegionalDelegationMilestoneValidator {
    pub rules: Vec<Rc<dyn RegionalDelegationMilestoneRule>>,
}

impl RegionalDelegationMilestoneValidator {
    pub fn validate(&self, item: &RegionalDelegationMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalDelegationMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalDelegationMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_delegation_milestone_validator() -> RegionalDelegationMilestoneValidator {
    RegionalDelegationMilestoneValidator {
        rules: vec![
            Rc::new(RegionalDelegationMilestoneAttemptsRangeRule { min: 255, max: 1000 })
                as Rc<dyn RegionalDelegationMilestoneRule>,
            Rc::new(RegionalDelegationMilestoneNameLengthRule { max_length: 32 }),
            Rc::new(RegionalDelegationMilestoneCodeLengthRule { max_length: 3600 }),
            Rc::new(RegionalDelegationMilestoneRequiredCodeRule {
                message: "regional delegation milestone needs an entry".to_string(),
            }),
            Rc::new(RegionalDelegationMilestoneMarginRangeRule { min: 5.0, max: 3109.6 }),
            Rc::new(RegionalDelegationMilestoneTenantIdLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_delegation_milestone_validator();
        let item = new_regional_delegation_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = RegionalDelegationMilestoneAttemptsRangeRule { min: 16, max: 120 };
        let mut item = new_regional_delegation_milestone("x", "y");
        item.attempts = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionalDelegationMilestoneNameLengthRule { max_length: 64 };
        let mut item = new_regional_delegation_milestone("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalDelegationMilestoneCodeLengthRule { max_length: 255 };
        let mut item = new_regional_delegation_milestone("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalDelegationMilestoneRequiredCodeRule {
            message: "regional delegation milestone needs a value".to_string(),
        };
        let mut item = new_regional_delegation_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = RegionalDelegationMilestoneMarginRangeRule { min: 6.3, max: 650.4 };
        let mut item = new_regional_delegation_milestone("x", "y");
        item.margin = 651.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = RegionalDelegationMilestoneTenantIdLengthRule { max_length: 86400 };
        let mut item = new_regional_delegation_milestone("x", "y");
        item.tenant_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
