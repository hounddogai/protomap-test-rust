use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary verifier target calculation.
pub trait PrimaryVerifierTargetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryVerifierTarget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryVerifierTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryVerifierTargetFactor for FlaggedPrimaryVerifierTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryVerifierTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryVerifierTargetAdjuster {
    pub minimum: f64,
}

impl PrimaryVerifierTargetFactor for MinimumPrimaryVerifierTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryVerifierTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryVerifierTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryVerifierTargetFactor for TieredPrimaryVerifierTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryVerifierTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierTargetPortOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryVerifierTargetFactor for PrimaryVerifierTargetPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryVerifierTarget, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

/// Runs the primary verifier target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryVerifierTargetEngine {
    pub steps: Vec<Rc<dyn PrimaryVerifierTargetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryVerifierTargetEngine {
    pub fn run(&mut self, item: &PrimaryVerifierTarget) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryVerifierTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_verifier_target_engine(flags: &flags::Registry) -> PrimaryVerifierTargetEngine {
    PrimaryVerifierTargetEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryVerifierTargetAdjuster {
                flags: flags.clone(),
                flag: "primary-verifier-target-adaptive".to_string(),
                percent: -4.4,
            }) as Rc<dyn PrimaryVerifierTargetFactor>,
            Rc::new(MinimumPrimaryVerifierTargetAdjuster { minimum: 48.1 }),
            Rc::new(TieredPrimaryVerifierTargetAdjuster {
                bounds: vec![1813.0, 5442.0, 6363.0, 7419.0],
                rates: vec![0.87, 1.2, 1.09, 1.02],
            }),
            Rc::new(PrimaryVerifierTargetPortOffsetAdjuster { weight: 1.44 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 31374.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_verifier_target_engine(&flags::new_registry());
        let item = new_primary_verifier_target("code-657", "name-40");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
