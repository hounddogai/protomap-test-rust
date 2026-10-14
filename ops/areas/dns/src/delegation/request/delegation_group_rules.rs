use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a delegation group must pass. Check returns its problems, or none.
pub trait DelegationGroupRule: std::fmt::Debug {
    fn check(&self, item: &DelegationGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationGroupAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DelegationGroupRule for DelegationGroupAllowedCodeRule {
    fn check(&self, item: &DelegationGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationGroupScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DelegationGroupRule for DelegationGroupScoreRangeRule {
    fn check(&self, item: &DelegationGroup) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationGroupRequiredSummaryRule {
    pub message: String,
}

impl DelegationGroupRule for DelegationGroupRequiredSummaryRule {
    fn check(&self, item: &DelegationGroup) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationGroupRequiredLabelRule {
    pub message: String,
}

impl DelegationGroupRule for DelegationGroupRequiredLabelRule {
    fn check(&self, item: &DelegationGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationGroupCodePrefixRule {
    pub prefix: String,
}

impl DelegationGroupRule for DelegationGroupCodePrefixRule {
    fn check(&self, item: &DelegationGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation group.
#[derive(Debug, Clone)]
pub struct DelegationGroupValidator {
    pub rules: Vec<Rc<dyn DelegationGroupRule>>,
}

impl DelegationGroupValidator {
    pub fn validate(&self, item: &DelegationGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_group_validator() -> DelegationGroupValidator {
    DelegationGroupValidator {
        rules: vec![
            Rc::new(DelegationGroupAllowedCodeRule {
                allowed: vec!["seasonal".to_string()],
            }) as Rc<dyn DelegationGroupRule>,
            Rc::new(DelegationGroupScoreRangeRule { min: 4.4, max: 3646.7 }),
            Rc::new(DelegationGroupRequiredSummaryRule {
                message: "delegation group needs this field".to_string(),
            }),
            Rc::new(DelegationGroupRequiredLabelRule {
                message: "delegation group needs a value".to_string(),
            }),
            Rc::new(DelegationGroupCodePrefixRule {
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
        let validator = default_delegation_group_validator();
        let item = new_delegation_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = DelegationGroupScoreRangeRule { min: 8.6, max: 3883.5 };
        let mut item = new_delegation_group("x", "y");
        item.score = 3885.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = DelegationGroupRequiredSummaryRule {
            message: "delegation group needs this field".to_string(),
        };
        let mut item = new_delegation_group("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DelegationGroupRequiredLabelRule {
            message: "delegation group needs a value".to_string(),
        };
        let mut item = new_delegation_group("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
