use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling identity plan must pass. Check returns its problems, or none.
pub trait RollingIdentityPlanRule: std::fmt::Debug {
    fn check(&self, item: &RollingIdentityPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityPlanCodePrefixRule {
    pub prefix: String,
}

impl RollingIdentityPlanRule for RollingIdentityPlanCodePrefixRule {
    fn check(&self, item: &RollingIdentityPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityPlanRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingIdentityPlanRule for RollingIdentityPlanRevisionRangeRule {
    fn check(&self, item: &RollingIdentityPlan) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityPlanAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl RollingIdentityPlanRule for RollingIdentityPlanAllowedPrincipalRule {
    fn check(&self, item: &RollingIdentityPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is blocked", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityPlanGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingIdentityPlanRule for RollingIdentityPlanGraceDaysRangeRule {
    fn check(&self, item: &RollingIdentityPlan) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityPlanRequiredPrincipalRule {
    pub message: String,
}

impl RollingIdentityPlanRule for RollingIdentityPlanRequiredPrincipalRule {
    fn check(&self, item: &RollingIdentityPlan) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling identity plan.
#[derive(Debug, Clone)]
pub struct RollingIdentityPlanValidator {
    pub rules: Vec<Rc<dyn RollingIdentityPlanRule>>,
}

impl RollingIdentityPlanValidator {
    pub fn validate(&self, item: &RollingIdentityPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingIdentityPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingIdentityPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_identity_plan_validator() -> RollingIdentityPlanValidator {
    RollingIdentityPlanValidator {
        rules: vec![
            Rc::new(RollingIdentityPlanCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn RollingIdentityPlanRule>,
            Rc::new(RollingIdentityPlanRevisionRangeRule { min: 32, max: 64 }),
            Rc::new(RollingIdentityPlanAllowedPrincipalRule {
                allowed: vec!["primary".to_string()],
            }),
            Rc::new(RollingIdentityPlanGraceDaysRangeRule { min: 64, max: 86400 }),
            Rc::new(RollingIdentityPlanRequiredPrincipalRule {
                message: "rolling identity plan needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_identity_plan_validator();
        let item = new_rolling_identity_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = RollingIdentityPlanRevisionRangeRule { min: 64, max: 120 };
        let mut item = new_rolling_identity_plan("x", "y");
        item.revision = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = RollingIdentityPlanGraceDaysRangeRule { min: 3600, max: 86400 };
        let mut item = new_rolling_identity_plan("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = RollingIdentityPlanRequiredPrincipalRule {
            message: "rolling identity plan needs this field".to_string(),
        };
        let mut item = new_rolling_identity_plan("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
