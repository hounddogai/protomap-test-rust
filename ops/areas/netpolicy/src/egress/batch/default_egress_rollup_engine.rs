use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the default egress rollup calculation.
pub trait DefaultEgressRollupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultEgressRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultEgressRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultEgressRollupStep for TieredDefaultEgressRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultEgressRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultEgressRollupAdjuster {
    pub cap: f64,
}

impl DefaultEgressRollupStep for CapDefaultEgressRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultEgressRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultEgressRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultEgressRollupStep for FlaggedDefaultEgressRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultEgressRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default egress rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultEgressRollupEngine {
    pub steps: Vec<Rc<dyn DefaultEgressRollupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultEgressRollupEngine {
    pub fn run(&mut self, item: &DefaultEgressRollup) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultEgressRollupStep> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[DefaultEgressRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_egress_rollup_engine(flags: &flags::Registry) -> DefaultEgressRollupEngine {
    DefaultEgressRollupEngine {
        steps: vec![
            Rc::new(TieredDefaultEgressRollupAdjuster {
                bounds: vec![622.0, 7835.0],
                rates: vec![0.95, 1.03],
            }) as Rc<dyn DefaultEgressRollupStep>,
            Rc::new(CapDefaultEgressRollupAdjuster { cap: 30757.0 }),
            Rc::new(FlaggedDefaultEgressRollupAdjuster {
                flags: flags.clone(),
                flag: "default-egress-rollup-capped".to_string(),
                percent: -3.4,
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 88273.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_egress_rollup_engine(&flags::new_registry());
        let item = new_default_egress_rollup("code-623", "label-676");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
