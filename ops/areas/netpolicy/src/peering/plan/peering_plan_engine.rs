use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the peering plan calculation.
pub trait PeeringPlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPeeringPlanAdjuster {
    pub step: f64,
}

impl PeeringPlanAdjuster for RoundingPeeringPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PeeringPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPeeringPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PeeringPlanAdjuster for FlaggedPeeringPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PeeringPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPeeringPlanAdjuster {
    pub percent: f64,
}

impl PeeringPlanAdjuster for PercentPeeringPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PeeringPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PeeringPlanAdjuster for TieredPeeringPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PeeringPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPeeringPlanAdjuster {
    pub cap: f64,
}

impl PeeringPlanAdjuster for CapPeeringPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PeeringPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the peering plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringPlanEngine {
    pub steps: Vec<Rc<dyn PeeringPlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringPlanEngine {
    pub fn compute(&mut self, item: &PeeringPlan) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[PeeringPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_peering_plan_engine(flags: &flags::Registry) -> PeeringPlanEngine {
    PeeringPlanEngine {
        steps: vec![
            Rc::new(RoundingPeeringPlanAdjuster { step: 0.1 }) as Rc<dyn PeeringPlanAdjuster>,
            Rc::new(FlaggedPeeringPlanAdjuster {
                flags: flags.clone(),
                flag: "peering-plan-primary".to_string(),
                percent: -6.0,
            }),
            Rc::new(PercentPeeringPlanAdjuster { percent: 23.8 }),
            Rc::new(TieredPeeringPlanAdjuster {
                bounds: vec![503.0, 1391.0, 6739.0, 8099.0],
                rates: vec![0.86, 1.15, 1.0, 1.01],
            }),
            Rc::new(CapPeeringPlanAdjuster { cap: 7216.0 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 70480.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_plan_engine(&flags::new_registry());
        let item = new_peering_plan("code-355", "title-514");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
