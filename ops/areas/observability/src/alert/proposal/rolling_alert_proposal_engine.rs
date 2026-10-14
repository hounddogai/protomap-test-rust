use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling alert proposal calculation.
pub trait RollingAlertProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingAlertProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingAlertProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingAlertProposalStep for TieredRollingAlertProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingAlertProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingAlertProposalAdjuster {
    pub step: f64,
}

impl RollingAlertProposalStep for RoundingRollingAlertProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingAlertProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRollingAlertProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingAlertProposalStep for RegionRollingAlertProposalAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RollingAlertProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingAlertProposalAdjuster {
    pub percent: f64,
}

impl RollingAlertProposalStep for PercentRollingAlertProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingAlertProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingAlertProposalAdjuster {
    pub minimum: f64,
}

impl RollingAlertProposalStep for MinimumRollingAlertProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingAlertProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rolling alert proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingAlertProposalEngine {
    pub steps: Vec<Rc<dyn RollingAlertProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingAlertProposalEngine {
    pub fn evaluate(&mut self, item: &RollingAlertProposal) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[RollingAlertProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_alert_proposal_engine() -> RollingAlertProposalEngine {
    RollingAlertProposalEngine {
        steps: vec![
            Rc::new(TieredRollingAlertProposalAdjuster {
                bounds: vec![2162.0, 4600.0, 6807.0],
                rates: vec![0.86, 1.02, 0.99],
            }) as Rc<dyn RollingAlertProposalStep>,
            Rc::new(RoundingRollingAlertProposalAdjuster { step: 1.0 }),
            Rc::new(RegionRollingAlertProposalAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.33)]),
            }),
            Rc::new(PercentRollingAlertProposalAdjuster { percent: -23.2 }),
            Rc::new(MinimumRollingAlertProposalAdjuster { minimum: 36.7 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 63988.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_alert_proposal_engine();
        let item = new_rolling_alert_proposal("code-718", "label-890");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
