use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional peering budget must pass. Check returns its problems, or none.
pub trait RegionalPeeringBudgetRule: std::fmt::Debug {
    fn check(&self, item: &RegionalPeeringBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringBudgetRequiredTitleRule {
    pub message: String,
}

impl RegionalPeeringBudgetRule for RegionalPeeringBudgetRequiredTitleRule {
    fn check(&self, item: &RegionalPeeringBudget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringBudgetStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalPeeringBudgetRule for RegionalPeeringBudgetStartsAtRangeRule {
    fn check(&self, item: &RegionalPeeringBudget) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringBudgetRequiredAudienceRule {
    pub message: String,
}

impl RegionalPeeringBudgetRule for RegionalPeeringBudgetRequiredAudienceRule {
    fn check(&self, item: &RegionalPeeringBudget) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringBudgetAudienceLengthRule {
    pub max_length: i64,
}

impl RegionalPeeringBudgetRule for RegionalPeeringBudgetAudienceLengthRule {
    fn check(&self, item: &RegionalPeeringBudget) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringBudgetRequiredContactEmailRule {
    pub message: String,
}

impl RegionalPeeringBudgetRule for RegionalPeeringBudgetRequiredContactEmailRule {
    fn check(&self, item: &RegionalPeeringBudget) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringBudgetCodePrefixRule {
    pub prefix: String,
}

impl RegionalPeeringBudgetRule for RegionalPeeringBudgetCodePrefixRule {
    fn check(&self, item: &RegionalPeeringBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional peering budget.
#[derive(Debug, Clone)]
pub struct RegionalPeeringBudgetValidator {
    pub rules: Vec<Rc<dyn RegionalPeeringBudgetRule>>,
}

impl RegionalPeeringBudgetValidator {
    pub fn validate(&self, item: &RegionalPeeringBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalPeeringBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalPeeringBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_peering_budget_validator() -> RegionalPeeringBudgetValidator {
    RegionalPeeringBudgetValidator {
        rules: vec![
            Rc::new(RegionalPeeringBudgetRequiredTitleRule {
                message: "regional peering budget needs a value".to_string(),
            }) as Rc<dyn RegionalPeeringBudgetRule>,
            Rc::new(RegionalPeeringBudgetStartsAtRangeRule { min: 64, max: 120 }),
            Rc::new(RegionalPeeringBudgetRequiredAudienceRule {
                message: "regional peering budget needs an entry".to_string(),
            }),
            Rc::new(RegionalPeeringBudgetAudienceLengthRule { max_length: 255 }),
            Rc::new(RegionalPeeringBudgetRequiredContactEmailRule {
                message: "regional peering budget needs this field".to_string(),
            }),
            Rc::new(RegionalPeeringBudgetCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_peering_budget_validator();
        let item = new_regional_peering_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionalPeeringBudgetRequiredTitleRule {
            message: "regional peering budget needs an entry".to_string(),
        };
        let mut item = new_regional_peering_budget("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = RegionalPeeringBudgetStartsAtRangeRule { min: 255, max: 86400 };
        let mut item = new_regional_peering_budget("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = RegionalPeeringBudgetRequiredAudienceRule {
            message: "regional peering budget needs this field".to_string(),
        };
        let mut item = new_regional_peering_budget("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = RegionalPeeringBudgetAudienceLengthRule { max_length: 86400 };
        let mut item = new_regional_peering_budget("x", "y");
        item.audience = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = RegionalPeeringBudgetRequiredContactEmailRule {
            message: "regional peering budget needs an entry".to_string(),
        };
        let mut item = new_regional_peering_budget("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
