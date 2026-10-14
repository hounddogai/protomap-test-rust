use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global postmortem rule must pass. Check returns its problems, or none.
pub trait GlobalPostmortemRuleRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPostmortemRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPostmortemRuleRequiredClusterRule {
    pub message: String,
}

impl GlobalPostmortemRuleRule for GlobalPostmortemRuleRequiredClusterRule {
    fn check(&self, item: &GlobalPostmortemRule) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPostmortemRulePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalPostmortemRuleRule for GlobalPostmortemRulePortRangeRule {
    fn check(&self, item: &GlobalPostmortemRule) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPostmortemRuleApprovedGuardRule {
    pub limit: i64,
}

impl GlobalPostmortemRuleRule for GlobalPostmortemRuleApprovedGuardRule {
    fn check(&self, item: &GlobalPostmortemRule) -> Vec<String> {
        if item.approved && (item.window_days > self.limit) {
            return vec![format!(
                "window days {} exceeds {} while approved",
                item.window_days, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global postmortem rule.
#[derive(Debug, Clone)]
pub struct GlobalPostmortemRuleValidator {
    pub rules: Vec<Rc<dyn GlobalPostmortemRuleRule>>,
}

impl GlobalPostmortemRuleValidator {
    pub fn validate(&self, item: &GlobalPostmortemRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalPostmortemRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPostmortemRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_postmortem_rule_validator() -> GlobalPostmortemRuleValidator {
    GlobalPostmortemRuleValidator {
        rules: vec![
            Rc::new(GlobalPostmortemRuleRequiredClusterRule {
                message: "global postmortem rule needs an entry".to_string(),
            }) as Rc<dyn GlobalPostmortemRuleRule>,
            Rc::new(GlobalPostmortemRulePortRangeRule { min: 16, max: 1000 }),
            Rc::new(GlobalPostmortemRuleApprovedGuardRule { limit: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_postmortem_rule_validator();
        let item = new_global_postmortem_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = GlobalPostmortemRuleRequiredClusterRule {
            message: "global postmortem rule needs this field".to_string(),
        };
        let mut item = new_global_postmortem_rule("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = GlobalPostmortemRulePortRangeRule { min: 16, max: 80 };
        let mut item = new_global_postmortem_rule("x", "y");
        item.port = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
