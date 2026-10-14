use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the adaptive verifier baseline calculation.
pub trait AdaptiveVerifierBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveVerifierBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveVerifierBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveVerifierBaselineStep for FlaggedAdaptiveVerifierBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveVerifierBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveVerifierBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveVerifierBaselineStep for TieredAdaptiveVerifierBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveVerifierBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveVerifierBaselineAdjuster {
    pub minimum: f64,
}

impl AdaptiveVerifierBaselineStep for MinimumAdaptiveVerifierBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveVerifierBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the adaptive verifier baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveVerifierBaselineEngine {
    pub steps: Vec<Rc<dyn AdaptiveVerifierBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveVerifierBaselineEngine {
    pub fn compute(&mut self, item: &AdaptiveVerifierBaseline) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveVerifierBaselineStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveVerifierBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_verifier_baseline_engine(flags: &flags::Registry) -> AdaptiveVerifierBaselineEngine {
    AdaptiveVerifierBaselineEngine {
        steps: vec![
            Rc::new(FlaggedAdaptiveVerifierBaselineAdjuster {
                flags: flags.clone(),
                flag: "adaptive-verifier-baseline-lenient".to_string(),
                percent: 1.4,
            }) as Rc<dyn AdaptiveVerifierBaselineStep>,
            Rc::new(TieredAdaptiveVerifierBaselineAdjuster {
                bounds: vec![3473.0, 4022.0, 6760.0],
                rates: vec![1.19, 0.98, 1.15],
            }),
            Rc::new(MinimumAdaptiveVerifierBaselineAdjuster { minimum: 45.9 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 86682.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_verifier_baseline_engine(&flags::new_registry());
        let item = new_adaptive_verifier_baseline("code-864", "title-688");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
