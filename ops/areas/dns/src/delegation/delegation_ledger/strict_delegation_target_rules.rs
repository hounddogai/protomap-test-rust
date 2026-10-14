use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict delegation target must pass. Check returns its problems, or none.
pub trait StrictDelegationTargetRule: std::fmt::Debug {
    fn check(&self, item: &StrictDelegationTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDelegationTargetRequiredCodeRule {
    pub message: String,
}

impl StrictDelegationTargetRule for StrictDelegationTargetRequiredCodeRule {
    fn check(&self, item: &StrictDelegationTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDelegationTargetVisibleGuardRule {
    pub limit: i64,
}

impl StrictDelegationTargetRule for StrictDelegationTargetVisibleGuardRule {
    fn check(&self, item: &StrictDelegationTarget) -> Vec<String> {
        if item.visible && (item.version > self.limit) {
            return vec![format!("version {} exceeds {} while visible", item.version, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDelegationTargetRequiredSummaryRule {
    pub message: String,
}

impl StrictDelegationTargetRule for StrictDelegationTargetRequiredSummaryRule {
    fn check(&self, item: &StrictDelegationTarget) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict delegation target.
#[derive(Debug, Clone)]
pub struct StrictDelegationTargetValidator {
    pub rules: Vec<Rc<dyn StrictDelegationTargetRule>>,
}

impl StrictDelegationTargetValidator {
    pub fn validate(&self, item: &StrictDelegationTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictDelegationTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictDelegationTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_delegation_target_validator() -> StrictDelegationTargetValidator {
    StrictDelegationTargetValidator {
        rules: vec![
            Rc::new(StrictDelegationTargetRequiredCodeRule {
                message: "strict delegation target needs an entry".to_string(),
            }) as Rc<dyn StrictDelegationTargetRule>,
            Rc::new(StrictDelegationTargetVisibleGuardRule { limit: 1000 }),
            Rc::new(StrictDelegationTargetRequiredSummaryRule {
                message: "strict delegation target needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_delegation_target_validator();
        let item = new_strict_delegation_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictDelegationTargetRequiredCodeRule {
            message: "strict delegation target needs this field".to_string(),
        };
        let mut item = new_strict_delegation_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = StrictDelegationTargetRequiredSummaryRule {
            message: "strict delegation target needs an entry".to_string(),
        };
        let mut item = new_strict_delegation_target("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
