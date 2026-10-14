use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient nonce tier calculation.
pub trait LenientNonceTierStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientNonceTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientNonceTierAdjuster {
    pub cap: f64,
}

impl LenientNonceTierStep for CapLenientNonceTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientNonceTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLenientNonceTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientNonceTierStep for LabelLenientNonceTierAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LenientNonceTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientNonceTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientNonceTierStep for TieredLenientNonceTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientNonceTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientNonceTierAdjuster {
    pub step: f64,
}

impl LenientNonceTierStep for RoundingLenientNonceTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientNonceTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the lenient nonce tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientNonceTierEngine {
    pub steps: Vec<Rc<dyn LenientNonceTierStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientNonceTierEngine {
    pub fn evaluate(&mut self, item: &LenientNonceTier) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[LenientNonceTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_nonce_tier_engine() -> LenientNonceTierEngine {
    LenientNonceTierEngine {
        steps: vec![
            Rc::new(CapLenientNonceTierAdjuster { cap: 24039.0 }) as Rc<dyn LenientNonceTierStep>,
            Rc::new(LabelLenientNonceTierAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.23),
                    ("online".to_string(), 0.96),
                    ("west".to_string(), 1.31),
                ]),
            }),
            Rc::new(TieredLenientNonceTierAdjuster {
                bounds: vec![196.0, 2736.0, 3254.0, 3842.0],
                rates: vec![1.0, 0.93, 0.95, 0.81],
            }),
            Rc::new(RoundingLenientNonceTierAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 78682.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_nonce_tier_engine();
        let item = new_lenient_nonce_tier("code-500", "label-704");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
