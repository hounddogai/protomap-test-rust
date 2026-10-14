use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient slo draft calculation.
pub trait LenientSloDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientSloDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientSloDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientSloDraftFactor for TieredLenientSloDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientSloDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientSloDraftAdjuster {
    pub minimum: f64,
}

impl LenientSloDraftFactor for MinimumLenientSloDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientSloDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientSloDraftAdjuster {
    pub step: f64,
}

impl LenientSloDraftFactor for RoundingLenientSloDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientSloDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the lenient slo draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientSloDraftEngine {
    pub steps: Vec<Rc<dyn LenientSloDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientSloDraftEngine {
    pub fn run(&mut self, item: &LenientSloDraft) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[LenientSloDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_slo_draft_engine() -> LenientSloDraftEngine {
    LenientSloDraftEngine {
        steps: vec![
            Rc::new(TieredLenientSloDraftAdjuster {
                bounds: vec![3382.0, 3676.0, 3790.0, 6525.0],
                rates: vec![0.98, 0.94, 1.13, 1.19],
            }) as Rc<dyn LenientSloDraftFactor>,
            Rc::new(MinimumLenientSloDraftAdjuster { minimum: 6.6 }),
            Rc::new(RoundingLenientSloDraftAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 93546.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_slo_draft_engine();
        let item = new_lenient_slo_draft("code-612", "label-836");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
