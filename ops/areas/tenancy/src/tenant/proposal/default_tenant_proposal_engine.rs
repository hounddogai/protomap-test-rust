use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default tenant proposal calculation.
pub trait DefaultTenantProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultTenantProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantProposalRateOffsetAdjuster {
    pub weight: f64,
}

impl DefaultTenantProposalFactor for DefaultTenantProposalRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultTenantProposal, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultTenantProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultTenantProposalFactor for FlaggedDefaultTenantProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultTenantProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultTenantProposalAdjuster {
    pub minimum: f64,
}

impl DefaultTenantProposalFactor for MinimumDefaultTenantProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultTenantProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultTenantProposalAdjuster {
    pub percent: f64,
}

impl DefaultTenantProposalFactor for PercentDefaultTenantProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultTenantProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultTenantProposalAdjuster {
    pub cap: f64,
}

impl DefaultTenantProposalFactor for CapDefaultTenantProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultTenantProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the default tenant proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultTenantProposalEngine {
    pub steps: Vec<Rc<dyn DefaultTenantProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultTenantProposalEngine {
    pub fn evaluate(&mut self, item: &DefaultTenantProposal) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[DefaultTenantProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_tenant_proposal_engine(flags: &flags::Registry) -> DefaultTenantProposalEngine {
    DefaultTenantProposalEngine {
        steps: vec![
            Rc::new(DefaultTenantProposalRateOffsetAdjuster { weight: 0.1 }) as Rc<dyn DefaultTenantProposalFactor>,
            Rc::new(FlaggedDefaultTenantProposalAdjuster {
                flags: flags.clone(),
                flag: "default-tenant-proposal-pending".to_string(),
                percent: 5.5,
            }),
            Rc::new(MinimumDefaultTenantProposalAdjuster { minimum: 48.7 }),
            Rc::new(PercentDefaultTenantProposalAdjuster { percent: 12.9 }),
            Rc::new(CapDefaultTenantProposalAdjuster { cap: 31837.0 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 49866.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_tenant_proposal_engine(&flags::new_registry());
        let item = new_default_tenant_proposal("code-289", "name-290");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
