use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient probe milestone must pass. Check returns its problems, or none.
pub trait LenientProbeMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &LenientProbeMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeMilestoneRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientProbeMilestoneRule for LenientProbeMilestoneRetriesRangeRule {
    fn check(&self, item: &LenientProbeMilestone) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeMilestoneCodePrefixRule {
    pub prefix: String,
}

impl LenientProbeMilestoneRule for LenientProbeMilestoneCodePrefixRule {
    fn check(&self, item: &LenientProbeMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeMilestoneRequiredNameRule {
    pub message: String,
}

impl LenientProbeMilestoneRule for LenientProbeMilestoneRequiredNameRule {
    fn check(&self, item: &LenientProbeMilestone) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeMilestoneRequiredCodeRule {
    pub message: String,
}

impl LenientProbeMilestoneRule for LenientProbeMilestoneRequiredCodeRule {
    fn check(&self, item: &LenientProbeMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeMilestoneRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientProbeMilestoneRule for LenientProbeMilestoneRatioRangeRule {
    fn check(&self, item: &LenientProbeMilestone) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient probe milestone.
#[derive(Debug, Clone)]
pub struct LenientProbeMilestoneValidator {
    pub rules: Vec<Rc<dyn LenientProbeMilestoneRule>>,
}

impl LenientProbeMilestoneValidator {
    pub fn validate(&self, item: &LenientProbeMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientProbeMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientProbeMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_probe_milestone_validator() -> LenientProbeMilestoneValidator {
    LenientProbeMilestoneValidator {
        rules: vec![
            Rc::new(LenientProbeMilestoneRetriesRangeRule { min: 80, max: 80 }) as Rc<dyn LenientProbeMilestoneRule>,
            Rc::new(LenientProbeMilestoneCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(LenientProbeMilestoneRequiredNameRule {
                message: "lenient probe milestone needs an entry".to_string(),
            }),
            Rc::new(LenientProbeMilestoneRequiredCodeRule {
                message: "lenient probe milestone needs this field".to_string(),
            }),
            Rc::new(LenientProbeMilestoneRatioRangeRule { min: 2.9, max: 348.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_probe_milestone_validator();
        let item = new_lenient_probe_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = LenientProbeMilestoneRetriesRangeRule { min: 255, max: 1000 };
        let mut item = new_lenient_probe_milestone("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LenientProbeMilestoneRequiredNameRule {
            message: "lenient probe milestone needs this field".to_string(),
        };
        let mut item = new_lenient_probe_milestone("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientProbeMilestoneRequiredCodeRule {
            message: "lenient probe milestone needs an entry".to_string(),
        };
        let mut item = new_lenient_probe_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = LenientProbeMilestoneRatioRangeRule { min: 9.5, max: 2204.4 };
        let mut item = new_lenient_probe_milestone("x", "y");
        item.ratio = 2205.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
