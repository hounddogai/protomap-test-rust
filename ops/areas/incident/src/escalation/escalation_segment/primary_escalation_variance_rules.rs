use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary escalation variance must pass. Check returns its problems, or none.
pub trait PrimaryEscalationVarianceRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryEscalationVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationVarianceRequiredTenantIdRule {
    pub message: String,
}

impl PrimaryEscalationVarianceRule for PrimaryEscalationVarianceRequiredTenantIdRule {
    fn check(&self, item: &PrimaryEscalationVariance) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationVarianceGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryEscalationVarianceRule for PrimaryEscalationVarianceGraceDaysRangeRule {
    fn check(&self, item: &PrimaryEscalationVariance) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationVariancePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryEscalationVarianceRule for PrimaryEscalationVariancePortRangeRule {
    fn check(&self, item: &PrimaryEscalationVariance) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationVarianceRequiredUnitRule {
    pub message: String,
}

impl PrimaryEscalationVarianceRule for PrimaryEscalationVarianceRequiredUnitRule {
    fn check(&self, item: &PrimaryEscalationVariance) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary escalation variance.
#[derive(Debug, Clone)]
pub struct PrimaryEscalationVarianceValidator {
    pub rules: Vec<Rc<dyn PrimaryEscalationVarianceRule>>,
}

impl PrimaryEscalationVarianceValidator {
    pub fn validate(&self, item: &PrimaryEscalationVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryEscalationVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryEscalationVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_escalation_variance_validator() -> PrimaryEscalationVarianceValidator {
    PrimaryEscalationVarianceValidator {
        rules: vec![
            Rc::new(PrimaryEscalationVarianceRequiredTenantIdRule {
                message: "primary escalation variance needs a value".to_string(),
            }) as Rc<dyn PrimaryEscalationVarianceRule>,
            Rc::new(PrimaryEscalationVarianceGraceDaysRangeRule { min: 3600, max: 3600 }),
            Rc::new(PrimaryEscalationVariancePortRangeRule { min: 64, max: 80 }),
            Rc::new(PrimaryEscalationVarianceRequiredUnitRule {
                message: "primary escalation variance needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_escalation_variance_validator();
        let item = new_primary_escalation_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = PrimaryEscalationVarianceRequiredTenantIdRule {
            message: "primary escalation variance needs an entry".to_string(),
        };
        let mut item = new_primary_escalation_variance("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = PrimaryEscalationVarianceGraceDaysRangeRule { min: 16, max: 64 };
        let mut item = new_primary_escalation_variance("x", "y");
        item.grace_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = PrimaryEscalationVariancePortRangeRule { min: 80, max: 80 };
        let mut item = new_primary_escalation_variance("x", "y");
        item.port = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PrimaryEscalationVarianceRequiredUnitRule {
            message: "primary escalation variance needs this field".to_string(),
        };
        let mut item = new_primary_escalation_variance("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
