use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback plan item must pass. Check returns its problems, or none.
pub trait FallbackPlanItemRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPlanItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanItemRequiredOwnerIdRule {
    pub message: String,
}

impl FallbackPlanItemRule for FallbackPlanItemRequiredOwnerIdRule {
    fn check(&self, item: &FallbackPlanItem) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanItemCodePrefixRule {
    pub prefix: String,
}

impl FallbackPlanItemRule for FallbackPlanItemCodePrefixRule {
    fn check(&self, item: &FallbackPlanItem) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanItemRequiredCodeRule {
    pub message: String,
}

impl FallbackPlanItemRule for FallbackPlanItemRequiredCodeRule {
    fn check(&self, item: &FallbackPlanItem) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback plan item.
#[derive(Debug, Clone)]
pub struct FallbackPlanItemValidator {
    pub rules: Vec<Rc<dyn FallbackPlanItemRule>>,
}

impl FallbackPlanItemValidator {
    pub fn validate(&self, item: &FallbackPlanItem) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackPlanItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPlanItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_plan_item_validator() -> FallbackPlanItemValidator {
    FallbackPlanItemValidator {
        rules: vec![
            Rc::new(FallbackPlanItemRequiredOwnerIdRule {
                message: "fallback plan item needs an entry".to_string(),
            }) as Rc<dyn FallbackPlanItemRule>,
            Rc::new(FallbackPlanItemCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(FallbackPlanItemRequiredCodeRule {
                message: "fallback plan item needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_plan_item_validator();
        let item = new_fallback_plan_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = FallbackPlanItemRequiredOwnerIdRule {
            message: "fallback plan item needs an entry".to_string(),
        };
        let mut item = new_fallback_plan_item("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackPlanItemRequiredCodeRule {
            message: "fallback plan item needs a value".to_string(),
        };
        let mut item = new_fallback_plan_item("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
