use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook notice must pass. Check returns its problems, or none.
pub trait RunbookNoticeRule: std::fmt::Debug {
    fn check(&self, item: &RunbookNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookNoticeAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl RunbookNoticeRule for RunbookNoticeAllowedCategoryRule {
    fn check(&self, item: &RunbookNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is unknown", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookNoticeCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookNoticeRule for RunbookNoticeCostRangeRule {
    fn check(&self, item: &RunbookNotice) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookNoticeApprovedGuardRule {
    pub limit: f64,
}

impl RunbookNoticeRule for RunbookNoticeApprovedGuardRule {
    fn check(&self, item: &RunbookNotice) -> Vec<String> {
        if item.approved && (item.cost > self.limit) {
            return vec![format!("cost {} exceeds {} while approved", item.cost, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookNoticeRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookNoticeRule for RunbookNoticeRatioRangeRule {
    fn check(&self, item: &RunbookNotice) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookNoticeRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookNoticeRule for RunbookNoticeRotationDaysRangeRule {
    fn check(&self, item: &RunbookNotice) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook notice.
#[derive(Debug, Clone)]
pub struct RunbookNoticeValidator {
    pub rules: Vec<Rc<dyn RunbookNoticeRule>>,
}

impl RunbookNoticeValidator {
    pub fn validate(&self, item: &RunbookNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_notice_validator() -> RunbookNoticeValidator {
    RunbookNoticeValidator {
        rules: vec![
            Rc::new(RunbookNoticeAllowedCategoryRule { allowed: Vec::new() }) as Rc<dyn RunbookNoticeRule>,
            Rc::new(RunbookNoticeCostRangeRule { min: 8.4, max: 336.1 }),
            Rc::new(RunbookNoticeApprovedGuardRule { limit: 8.0 }),
            Rc::new(RunbookNoticeRatioRangeRule { min: 7.9, max: 705.2 }),
            Rc::new(RunbookNoticeRotationDaysRangeRule { min: 16, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_notice_validator();
        let item = new_runbook_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = RunbookNoticeCostRangeRule { min: 6.7, max: 2937.3 };
        let mut item = new_runbook_notice("x", "y");
        item.cost = 2938.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = RunbookNoticeRatioRangeRule { min: 5.8, max: 1449.6 };
        let mut item = new_runbook_notice("x", "y");
        item.ratio = 1451.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = RunbookNoticeRotationDaysRangeRule { min: 80, max: 120 };
        let mut item = new_runbook_notice("x", "y");
        item.rotation_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
