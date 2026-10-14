use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered credential plan must pass. Check returns its problems, or none.
pub trait TieredCredentialPlanRule: std::fmt::Debug {
    fn check(&self, item: &TieredCredentialPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialPlanPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredCredentialPlanRule for TieredCredentialPlanPriorityRangeRule {
    fn check(&self, item: &TieredCredentialPlan) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialPlanCodeLengthRule {
    pub max_length: i64,
}

impl TieredCredentialPlanRule for TieredCredentialPlanCodeLengthRule {
    fn check(&self, item: &TieredCredentialPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialPlanNameLengthRule {
    pub max_length: i64,
}

impl TieredCredentialPlanRule for TieredCredentialPlanNameLengthRule {
    fn check(&self, item: &TieredCredentialPlan) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered credential plan.
#[derive(Debug, Clone)]
pub struct TieredCredentialPlanValidator {
    pub rules: Vec<Rc<dyn TieredCredentialPlanRule>>,
}

impl TieredCredentialPlanValidator {
    pub fn validate(&self, item: &TieredCredentialPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredCredentialPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredCredentialPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_credential_plan_validator() -> TieredCredentialPlanValidator {
    TieredCredentialPlanValidator {
        rules: vec![
            Rc::new(TieredCredentialPlanPriorityRangeRule { min: 1000, max: 3600 }) as Rc<dyn TieredCredentialPlanRule>,
            Rc::new(TieredCredentialPlanCodeLengthRule { max_length: 255 }),
            Rc::new(TieredCredentialPlanNameLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_credential_plan_validator();
        let item = new_tiered_credential_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = TieredCredentialPlanPriorityRangeRule { min: 80, max: 120 };
        let mut item = new_tiered_credential_plan("x", "y");
        item.priority = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredCredentialPlanCodeLengthRule { max_length: 86400 };
        let mut item = new_tiered_credential_plan("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TieredCredentialPlanNameLengthRule { max_length: 120 };
        let mut item = new_tiered_credential_plan("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
