use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient allowlist estimate calculation.
pub trait LenientAllowlistEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientAllowlistEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientAllowlistEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientAllowlistEstimateModifier for FlaggedLenientAllowlistEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientAllowlistEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientAllowlistEstimateAdjuster {
    pub step: f64,
}

impl LenientAllowlistEstimateModifier for RoundingLenientAllowlistEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientAllowlistEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientAllowlistEstimateAdjuster {
    pub cap: f64,
}

impl LenientAllowlistEstimateModifier for CapLenientAllowlistEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientAllowlistEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the lenient allowlist estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientAllowlistEstimateEngine {
    pub steps: Vec<Rc<dyn LenientAllowlistEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientAllowlistEstimateEngine {
    pub fn evaluate(&mut self, item: &LenientAllowlistEstimate) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientAllowlistEstimateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientAllowlistEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_allowlist_estimate_engine(flags: &flags::Registry) -> LenientAllowlistEstimateEngine {
    LenientAllowlistEstimateEngine {
        steps: vec![
            Rc::new(FlaggedLenientAllowlistEstimateAdjuster {
                flags: flags.clone(),
                flag: "lenient-allowlist-estimate-capped".to_string(),
                percent: -2.5,
            }) as Rc<dyn LenientAllowlistEstimateModifier>,
            Rc::new(RoundingLenientAllowlistEstimateAdjuster { step: 0.5 }),
            Rc::new(CapLenientAllowlistEstimateAdjuster { cap: 18325.0 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 51651.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_allowlist_estimate_engine(&flags::new_registry());
        let item = new_lenient_allowlist_estimate("code-802", "label-158");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
