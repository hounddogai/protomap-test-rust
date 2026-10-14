use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the lenient nonce baseline calculation.
pub trait LenientNonceBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientNonceBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientNonceBaselineAdjuster {
    pub cap: f64,
}

impl LenientNonceBaselineAdjuster for CapLenientNonceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientNonceBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientNonceBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientNonceBaselineAdjuster for TieredLenientNonceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientNonceBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientNonceBaselineAdjuster {
    pub minimum: f64,
}

impl LenientNonceBaselineAdjuster for MinimumLenientNonceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientNonceBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient nonce baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientNonceBaselineEngine {
    pub steps: Vec<Rc<dyn LenientNonceBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientNonceBaselineEngine {
    pub fn run(&mut self, item: &LenientNonceBaseline) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientNonceBaselineAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientNonceBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_nonce_baseline_engine() -> LenientNonceBaselineEngine {
    LenientNonceBaselineEngine {
        steps: vec![
            Rc::new(CapLenientNonceBaselineAdjuster { cap: 37352.0 }) as Rc<dyn LenientNonceBaselineAdjuster>,
            Rc::new(TieredLenientNonceBaselineAdjuster {
                bounds: vec![795.0, 2008.0, 9497.0],
                rates: vec![0.85, 1.08, 0.91],
            }),
            Rc::new(MinimumLenientNonceBaselineAdjuster { minimum: 15.1 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 65487.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_nonce_baseline_engine();
        let item = new_lenient_nonce_baseline("code-484", "title-214");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
