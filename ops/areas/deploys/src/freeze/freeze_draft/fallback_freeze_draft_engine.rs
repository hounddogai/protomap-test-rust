use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback freeze draft calculation.
pub trait FallbackFreezeDraftAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackFreezeDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackFreezeDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackFreezeDraftAdjuster for TieredFallbackFreezeDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackFreezeDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackFreezeDraftAdjuster {
    pub minimum: f64,
}

impl FallbackFreezeDraftAdjuster for MinimumFallbackFreezeDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackFreezeDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackFreezeDraftAdjuster {
    pub step: f64,
}

impl FallbackFreezeDraftAdjuster for RoundingFallbackFreezeDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackFreezeDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackFreezeDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackFreezeDraftAdjuster for FlaggedFallbackFreezeDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackFreezeDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback freeze draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackFreezeDraftEngine {
    pub steps: Vec<Rc<dyn FallbackFreezeDraftAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackFreezeDraftEngine {
    pub fn compute(&mut self, item: &FallbackFreezeDraft) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[FallbackFreezeDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_freeze_draft_engine(flags: &flags::Registry) -> FallbackFreezeDraftEngine {
    FallbackFreezeDraftEngine {
        steps: vec![
            Rc::new(TieredFallbackFreezeDraftAdjuster {
                bounds: vec![1701.0, 6884.0, 6902.0, 9702.0],
                rates: vec![0.9, 0.83, 0.83, 1.02],
            }) as Rc<dyn FallbackFreezeDraftAdjuster>,
            Rc::new(MinimumFallbackFreezeDraftAdjuster { minimum: 46.5 }),
            Rc::new(RoundingFallbackFreezeDraftAdjuster { step: 0.25 }),
            Rc::new(FlaggedFallbackFreezeDraftAdjuster {
                flags: flags.clone(),
                flag: "fallback-freeze-draft-global".to_string(),
                percent: -2.3,
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 70353.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_freeze_draft_engine(&flags::new_registry());
        let item = new_fallback_freeze_draft("code-391", "name-668");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
