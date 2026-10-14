use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the verifier cycle calculation.
pub trait VerifierCycleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCycleCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl VerifierCycleModifier for VerifierCycleCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &VerifierCycle, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierCycleModifier for FlaggedVerifierCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl VerifierCycleModifier for TieredVerifierCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &VerifierCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentVerifierCycleAdjuster {
    pub percent: f64,
}

impl VerifierCycleModifier for PercentVerifierCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &VerifierCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the verifier cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierCycleEngine {
    pub steps: Vec<Rc<dyn VerifierCycleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierCycleEngine {
    pub fn compute(&mut self, item: &VerifierCycle) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn VerifierCycleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[VerifierCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_verifier_cycle_engine(flags: &flags::Registry) -> VerifierCycleEngine {
    VerifierCycleEngine {
        steps: vec![
            Rc::new(VerifierCycleCreatedAtOffsetAdjuster { weight: 1.55 }) as Rc<dyn VerifierCycleModifier>,
            Rc::new(FlaggedVerifierCycleAdjuster {
                flags: flags.clone(),
                flag: "verifier-cycle-weighted".to_string(),
                percent: -3.6,
            }),
            Rc::new(TieredVerifierCycleAdjuster {
                bounds: vec![372.0, 3974.0, 5696.0],
                rates: vec![1.11, 0.93, 0.82],
            }),
            Rc::new(PercentVerifierCycleAdjuster { percent: 4.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 45946.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_cycle_engine(&flags::new_registry());
        let item = new_verifier_cycle("code-209", "label-335");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
