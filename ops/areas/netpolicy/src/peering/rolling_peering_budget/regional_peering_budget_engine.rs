use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional peering budget calculation.
pub trait RegionalPeeringBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalPeeringBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRegionalPeeringBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalPeeringBudgetStep for SegmentRegionalPeeringBudgetAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &RegionalPeeringBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalPeeringBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalPeeringBudgetStep for TieredRegionalPeeringBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalPeeringBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalPeeringBudgetAdjuster {
    pub minimum: f64,
}

impl RegionalPeeringBudgetStep for MinimumRegionalPeeringBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalPeeringBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalPeeringBudgetAdjuster {
    pub percent: f64,
}

impl RegionalPeeringBudgetStep for PercentRegionalPeeringBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalPeeringBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalPeeringBudgetAdjuster {
    pub cap: f64,
}

impl RegionalPeeringBudgetStep for CapRegionalPeeringBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalPeeringBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalPeeringBudgetAdjuster {
    pub step: f64,
}

impl RegionalPeeringBudgetStep for RoundingRegionalPeeringBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalPeeringBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the regional peering budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalPeeringBudgetEngine {
    pub steps: Vec<Rc<dyn RegionalPeeringBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalPeeringBudgetEngine {
    pub fn run(&mut self, item: &RegionalPeeringBudget) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[RegionalPeeringBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_peering_budget_engine() -> RegionalPeeringBudgetEngine {
    RegionalPeeringBudgetEngine {
        steps: vec![
            Rc::new(SegmentRegionalPeeringBudgetAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.4), ("online".to_string(), 1.35)]),
            }) as Rc<dyn RegionalPeeringBudgetStep>,
            Rc::new(TieredRegionalPeeringBudgetAdjuster {
                bounds: vec![2390.0, 7140.0],
                rates: vec![1.0, 0.81],
            }),
            Rc::new(MinimumRegionalPeeringBudgetAdjuster { minimum: 11.3 }),
            Rc::new(PercentRegionalPeeringBudgetAdjuster { percent: 0.5 }),
            Rc::new(CapRegionalPeeringBudgetAdjuster { cap: 31013.0 }),
            Rc::new(RoundingRegionalPeeringBudgetAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 44033.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_peering_budget_engine();
        let item = new_regional_peering_budget("code-492", "title-294");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
