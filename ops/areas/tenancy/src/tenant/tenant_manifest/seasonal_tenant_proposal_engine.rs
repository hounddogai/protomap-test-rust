use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal tenant proposal calculation.
pub trait SeasonalTenantProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalTenantProposal, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalTenantProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalTenantProposalStep for FlaggedSeasonalTenantProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalTenantProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSeasonalTenantProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalTenantProposalStep for NameSeasonalTenantProposalAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SeasonalTenantProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalTenantProposalAdjuster {
    pub minimum: f64,
}

impl SeasonalTenantProposalStep for MinimumSeasonalTenantProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalTenantProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalTenantProposalAdjuster {
    pub cap: f64,
}

impl SeasonalTenantProposalStep for CapSeasonalTenantProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalTenantProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalTenantProposalAdjuster {
    pub percent: f64,
}

impl SeasonalTenantProposalStep for PercentSeasonalTenantProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalTenantProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal tenant proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalTenantProposalEngine {
    pub steps: Vec<Rc<dyn SeasonalTenantProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalTenantProposalEngine {
    pub fn run(&mut self, item: &SeasonalTenantProposal) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[SeasonalTenantProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_tenant_proposal_engine(flags: &flags::Registry) -> SeasonalTenantProposalEngine {
    SeasonalTenantProposalEngine {
        steps: vec![
            Rc::new(FlaggedSeasonalTenantProposalAdjuster {
                flags: flags.clone(),
                flag: "seasonal-tenant-proposal-strict".to_string(),
                percent: 9.3,
            }) as Rc<dyn SeasonalTenantProposalStep>,
            Rc::new(NameSeasonalTenantProposalAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.18),
                    ("south".to_string(), 1.34),
                    ("online".to_string(), 1.16),
                ]),
            }),
            Rc::new(MinimumSeasonalTenantProposalAdjuster { minimum: 40.3 }),
            Rc::new(CapSeasonalTenantProposalAdjuster { cap: 47122.0 }),
            Rc::new(PercentSeasonalTenantProposalAdjuster { percent: 3.6 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 10660.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_tenant_proposal_engine(&flags::new_registry());
        let item = new_seasonal_tenant_proposal("code-725", "name-216");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
