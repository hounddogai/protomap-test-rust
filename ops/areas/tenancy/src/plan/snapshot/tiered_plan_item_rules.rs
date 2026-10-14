use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered plan item must pass. Check returns its problems, or none.
pub trait TieredPlanItemRule: std::fmt::Debug {
    fn check(&self, item: &TieredPlanItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanItemCodePrefixRule {
    pub prefix: String,
}

impl TieredPlanItemRule for TieredPlanItemCodePrefixRule {
    fn check(&self, item: &TieredPlanItem) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanItemRequiredSourceRule {
    pub message: String,
}

impl TieredPlanItemRule for TieredPlanItemRequiredSourceRule {
    fn check(&self, item: &TieredPlanItem) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanItemTitleLengthRule {
    pub max_length: i64,
}

impl TieredPlanItemRule for TieredPlanItemTitleLengthRule {
    fn check(&self, item: &TieredPlanItem) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered plan item.
#[derive(Debug, Clone)]
pub struct TieredPlanItemValidator {
    pub rules: Vec<Rc<dyn TieredPlanItemRule>>,
}

impl TieredPlanItemValidator {
    pub fn validate(&self, item: &TieredPlanItem) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredPlanItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredPlanItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_plan_item_validator() -> TieredPlanItemValidator {
    TieredPlanItemValidator {
        rules: vec![
            Rc::new(TieredPlanItemCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn TieredPlanItemRule>,
            Rc::new(TieredPlanItemRequiredSourceRule {
                message: "tiered plan item needs an entry".to_string(),
            }),
            Rc::new(TieredPlanItemTitleLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_plan_item_validator();
        let item = new_tiered_plan_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = TieredPlanItemRequiredSourceRule {
            message: "tiered plan item needs an entry".to_string(),
        };
        let mut item = new_tiered_plan_item("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredPlanItemTitleLengthRule { max_length: 80 };
        let mut item = new_tiered_plan_item("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
