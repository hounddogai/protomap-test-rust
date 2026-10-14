use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation metric calculation.
pub trait DelegationMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationMetricAdjuster {
    pub step: f64,
}

impl DelegationMetricStep for RoundingDelegationMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMetricRateOffsetAdjuster {
    pub weight: f64,
}

impl DelegationMetricStep for DelegationMetricRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &DelegationMetric, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationMetricStep for TieredDelegationMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationMetricAdjuster {
    pub cap: f64,
}

impl DelegationMetricStep for CapDelegationMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationMetricStep for FlaggedDelegationMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the delegation metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationMetricEngine {
    pub steps: Vec<Rc<dyn DelegationMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationMetricEngine {
    pub fn run(&mut self, item: &DelegationMetric) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DelegationMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_delegation_metric_engine(flags: &flags::Registry) -> DelegationMetricEngine {
    DelegationMetricEngine {
        steps: vec![
            Rc::new(RoundingDelegationMetricAdjuster { step: 1.0 }) as Rc<dyn DelegationMetricStep>,
            Rc::new(DelegationMetricRateOffsetAdjuster { weight: 1.14 }),
            Rc::new(TieredDelegationMetricAdjuster {
                bounds: vec![2879.0, 2919.0, 7728.0],
                rates: vec![0.84, 0.84, 0.84],
            }),
            Rc::new(CapDelegationMetricAdjuster { cap: 1268.0 }),
            Rc::new(FlaggedDelegationMetricAdjuster {
                flags: flags.clone(),
                flag: "delegation-metric-manual".to_string(),
                percent: 8.2,
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 37405.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_metric_engine(&flags::new_registry());
        let item = new_delegation_metric("code-178", "name-1");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
