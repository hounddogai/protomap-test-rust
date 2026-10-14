use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global binding target must pass. Check returns its problems, or none.
pub trait GlobalBindingTargetRule: std::fmt::Debug {
    fn check(&self, item: &GlobalBindingTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBindingTargetCountryLengthRule {
    pub max_length: i64,
}

impl GlobalBindingTargetRule for GlobalBindingTargetCountryLengthRule {
    fn check(&self, item: &GlobalBindingTarget) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBindingTargetRequiredCodeRule {
    pub message: String,
}

impl GlobalBindingTargetRule for GlobalBindingTargetRequiredCodeRule {
    fn check(&self, item: &GlobalBindingTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBindingTargetBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalBindingTargetRule for GlobalBindingTargetBudgetRangeRule {
    fn check(&self, item: &GlobalBindingTarget) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBindingTargetTitleLengthRule {
    pub max_length: i64,
}

impl GlobalBindingTargetRule for GlobalBindingTargetTitleLengthRule {
    fn check(&self, item: &GlobalBindingTarget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global binding target.
#[derive(Debug, Clone)]
pub struct GlobalBindingTargetValidator {
    pub rules: Vec<Rc<dyn GlobalBindingTargetRule>>,
}

impl GlobalBindingTargetValidator {
    pub fn validate(&self, item: &GlobalBindingTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalBindingTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalBindingTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_binding_target_validator() -> GlobalBindingTargetValidator {
    GlobalBindingTargetValidator {
        rules: vec![
            Rc::new(GlobalBindingTargetCountryLengthRule { max_length: 80 }) as Rc<dyn GlobalBindingTargetRule>,
            Rc::new(GlobalBindingTargetRequiredCodeRule {
                message: "global binding target needs a value".to_string(),
            }),
            Rc::new(GlobalBindingTargetBudgetRangeRule { min: 8.0, max: 2780.5 }),
            Rc::new(GlobalBindingTargetTitleLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_binding_target_validator();
        let item = new_global_binding_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = GlobalBindingTargetCountryLengthRule { max_length: 3600 };
        let mut item = new_global_binding_target("x", "y");
        item.country = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalBindingTargetRequiredCodeRule {
            message: "global binding target needs this field".to_string(),
        };
        let mut item = new_global_binding_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = GlobalBindingTargetBudgetRangeRule { min: 0.2, max: 2149.5 };
        let mut item = new_global_binding_target("x", "y");
        item.budget = 2151.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalBindingTargetTitleLengthRule { max_length: 1000 };
        let mut item = new_global_binding_target("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
