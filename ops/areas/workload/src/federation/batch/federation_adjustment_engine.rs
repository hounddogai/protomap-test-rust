use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation adjustment calculation.
pub trait FederationAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFederationAdjustmentAdjuster {
    pub percent: f64,
}

impl FederationAdjustmentFactor for PercentFederationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FederationAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationAdjustmentLimitOffsetAdjuster {
    pub weight: f64,
}

impl FederationAdjustmentFactor for FederationAdjustmentLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &FederationAdjustment, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationAdjustmentFactor for TieredFederationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFederationAdjustmentAdjuster {
    pub cap: f64,
}

impl FederationAdjustmentFactor for CapFederationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FederationAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationAdjustmentFactor for FlaggedFederationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationAdjustmentAdjuster {
    pub minimum: f64,
}

impl FederationAdjustmentFactor for MinimumFederationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the federation adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationAdjustmentEngine {
    pub steps: Vec<Rc<dyn FederationAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationAdjustmentEngine {
    pub fn price(&mut self, item: &FederationAdjustment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FederationAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_federation_adjustment_engine(flags: &flags::Registry) -> FederationAdjustmentEngine {
    FederationAdjustmentEngine {
        steps: vec![
            Rc::new(PercentFederationAdjustmentAdjuster { percent: 18.7 }) as Rc<dyn FederationAdjustmentFactor>,
            Rc::new(FederationAdjustmentLimitOffsetAdjuster { weight: 0.9 }),
            Rc::new(TieredFederationAdjustmentAdjuster {
                bounds: vec![911.0, 1888.0, 6997.0, 7455.0],
                rates: vec![1.08, 1.01, 1.02, 0.98],
            }),
            Rc::new(CapFederationAdjustmentAdjuster { cap: 41829.0 }),
            Rc::new(FlaggedFederationAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "federation-adjustment-strict".to_string(),
                percent: 1.7,
            }),
            Rc::new(MinimumFederationAdjustmentAdjuster { minimum: 26.3 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 99617.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_adjustment_engine(&flags::new_registry());
        let item = new_federation_adjustment("code-136", "title-142");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
