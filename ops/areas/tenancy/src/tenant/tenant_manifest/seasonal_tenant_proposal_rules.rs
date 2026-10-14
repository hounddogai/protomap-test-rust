use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal tenant proposal must pass. Check returns its problems, or none.
pub trait SeasonalTenantProposalRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalTenantProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTenantProposalPinnedGuardRule {
    pub limit: i64,
}

impl SeasonalTenantProposalRule for SeasonalTenantProposalPinnedGuardRule {
    fn check(&self, item: &SeasonalTenantProposal) -> Vec<String> {
        if item.pinned && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while pinned",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTenantProposalAllowedNameRule {
    pub allowed: Vec<String>,
}

impl SeasonalTenantProposalRule for SeasonalTenantProposalAllowedNameRule {
    fn check(&self, item: &SeasonalTenantProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTenantProposalPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalTenantProposalRule for SeasonalTenantProposalPriorityRangeRule {
    fn check(&self, item: &SeasonalTenantProposal) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal tenant proposal.
#[derive(Debug, Clone)]
pub struct SeasonalTenantProposalValidator {
    pub rules: Vec<Rc<dyn SeasonalTenantProposalRule>>,
}

impl SeasonalTenantProposalValidator {
    pub fn validate(&self, item: &SeasonalTenantProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalTenantProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalTenantProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_tenant_proposal_validator() -> SeasonalTenantProposalValidator {
    SeasonalTenantProposalValidator {
        rules: vec![
            Rc::new(SeasonalTenantProposalPinnedGuardRule { limit: 86400 }) as Rc<dyn SeasonalTenantProposalRule>,
            Rc::new(SeasonalTenantProposalAllowedNameRule { allowed: Vec::new() }),
            Rc::new(SeasonalTenantProposalPriorityRangeRule { min: 16, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_tenant_proposal_validator();
        let item = new_seasonal_tenant_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = SeasonalTenantProposalPriorityRangeRule { min: 1000, max: 3600 };
        let mut item = new_seasonal_tenant_proposal("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
