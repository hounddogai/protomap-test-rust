use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation estimate calculation.
pub trait DelegationEstimateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationEstimateFactor for FlaggedDelegationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationEstimateAdjuster {
    pub cap: f64,
}

impl DelegationEstimateFactor for CapDelegationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationEstimateFactor for TieredDelegationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationEstimateQuantityOffsetAdjuster {
    pub weight: f64,
}

impl DelegationEstimateFactor for DelegationEstimateQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &DelegationEstimate, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationEstimateAdjuster {
    pub minimum: f64,
}

impl DelegationEstimateFactor for MinimumDelegationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the delegation estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationEstimateEngine {
    pub steps: Vec<Rc<dyn DelegationEstimateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationEstimateEngine {
    pub fn compute(&mut self, item: &DelegationEstimate) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[DelegationEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_delegation_estimate_engine(flags: &flags::Registry) -> DelegationEstimateEngine {
    DelegationEstimateEngine {
        steps: vec![
            Rc::new(FlaggedDelegationEstimateAdjuster {
                flags: flags.clone(),
                flag: "delegation-estimate-pending".to_string(),
                percent: 7.9,
            }) as Rc<dyn DelegationEstimateFactor>,
            Rc::new(CapDelegationEstimateAdjuster { cap: 10794.0 }),
            Rc::new(TieredDelegationEstimateAdjuster {
                bounds: vec![233.0, 5670.0, 5842.0, 7211.0],
                rates: vec![0.96, 0.96, 0.83, 1.19],
            }),
            Rc::new(DelegationEstimateQuantityOffsetAdjuster { weight: 0.42 }),
            Rc::new(MinimumDelegationEstimateAdjuster { minimum: 42.5 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 79374.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_estimate_engine(&flags::new_registry());
        let item = new_delegation_estimate("code-248", "label-884");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
