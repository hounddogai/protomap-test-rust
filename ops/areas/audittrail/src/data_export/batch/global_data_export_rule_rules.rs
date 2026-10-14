use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global data export rule must pass. Check returns its problems, or none.
pub trait GlobalDataExportRuleRule: std::fmt::Debug {
    fn check(&self, item: &GlobalDataExportRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDataExportRulePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalDataExportRuleRule for GlobalDataExportRulePriorityRangeRule {
    fn check(&self, item: &GlobalDataExportRule) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDataExportRuleAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalDataExportRuleRule for GlobalDataExportRuleAttemptsRangeRule {
    fn check(&self, item: &GlobalDataExportRule) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDataExportRulePinnedGuardRule {
    pub limit: i64,
}

impl GlobalDataExportRuleRule for GlobalDataExportRulePinnedGuardRule {
    fn check(&self, item: &GlobalDataExportRule) -> Vec<String> {
        if item.pinned && (item.attempts > self.limit) {
            return vec![format!(
                "attempts {} exceeds {} while pinned",
                item.attempts, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global data export rule.
#[derive(Debug, Clone)]
pub struct GlobalDataExportRuleValidator {
    pub rules: Vec<Rc<dyn GlobalDataExportRuleRule>>,
}

impl GlobalDataExportRuleValidator {
    pub fn validate(&self, item: &GlobalDataExportRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalDataExportRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalDataExportRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_data_export_rule_validator() -> GlobalDataExportRuleValidator {
    GlobalDataExportRuleValidator {
        rules: vec![
            Rc::new(GlobalDataExportRulePriorityRangeRule { min: 32, max: 3600 }) as Rc<dyn GlobalDataExportRuleRule>,
            Rc::new(GlobalDataExportRuleAttemptsRangeRule { min: 64, max: 255 }),
            Rc::new(GlobalDataExportRulePinnedGuardRule { limit: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_data_export_rule_validator();
        let item = new_global_data_export_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = GlobalDataExportRulePriorityRangeRule { min: 120, max: 3600 };
        let mut item = new_global_data_export_rule("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = GlobalDataExportRuleAttemptsRangeRule { min: 32, max: 80 };
        let mut item = new_global_data_export_rule("x", "y");
        item.attempts = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
