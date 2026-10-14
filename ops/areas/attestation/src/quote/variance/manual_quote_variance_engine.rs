use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual quote variance calculation.
pub trait ManualQuoteVarianceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualQuoteVariance, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualQuoteVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualQuoteVarianceFactor for FlaggedManualQuoteVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualQuoteVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualQuoteVarianceAdjuster {
    pub cap: f64,
}

impl ManualQuoteVarianceFactor for CapManualQuoteVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualQuoteVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteVarianceMinimumOffsetAdjuster {
    pub weight: f64,
}

impl ManualQuoteVarianceFactor for ManualQuoteVarianceMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &ManualQuoteVariance, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualQuoteVarianceAdjuster {
    pub step: f64,
}

impl ManualQuoteVarianceFactor for RoundingManualQuoteVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualQuoteVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual quote variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualQuoteVarianceEngine {
    pub steps: Vec<Rc<dyn ManualQuoteVarianceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualQuoteVarianceEngine {
    pub fn evaluate(&mut self, item: &ManualQuoteVariance) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ManualQuoteVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_quote_variance_engine(flags: &flags::Registry) -> ManualQuoteVarianceEngine {
    ManualQuoteVarianceEngine {
        steps: vec![
            Rc::new(FlaggedManualQuoteVarianceAdjuster {
                flags: flags.clone(),
                flag: "manual-quote-variance-seasonal".to_string(),
                percent: -9.4,
            }) as Rc<dyn ManualQuoteVarianceFactor>,
            Rc::new(CapManualQuoteVarianceAdjuster { cap: 13401.0 }),
            Rc::new(ManualQuoteVarianceMinimumOffsetAdjuster { weight: 1.18 }),
            Rc::new(RoundingManualQuoteVarianceAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 74783.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_quote_variance_engine(&flags::new_registry());
        let item = new_manual_quote_variance("code-435", "name-976");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
