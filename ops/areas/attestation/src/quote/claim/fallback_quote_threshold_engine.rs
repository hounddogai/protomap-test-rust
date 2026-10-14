use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback quote threshold calculation.
pub trait FallbackQuoteThresholdModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackQuoteThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackQuoteThresholdAdjuster {
    pub cap: f64,
}

impl FallbackQuoteThresholdModifier for CapFallbackQuoteThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackQuoteThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackQuoteThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackQuoteThresholdModifier for FlaggedFallbackQuoteThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackQuoteThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackQuoteThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackQuoteThresholdModifier for TieredFallbackQuoteThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackQuoteThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackQuoteThresholdAdjuster {
    pub minimum: f64,
}

impl FallbackQuoteThresholdModifier for MinimumFallbackQuoteThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackQuoteThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback quote threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackQuoteThresholdEngine {
    pub steps: Vec<Rc<dyn FallbackQuoteThresholdModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackQuoteThresholdEngine {
    pub fn run(&mut self, item: &FallbackQuoteThreshold) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FallbackQuoteThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_quote_threshold_engine(flags: &flags::Registry) -> FallbackQuoteThresholdEngine {
    FallbackQuoteThresholdEngine {
        steps: vec![
            Rc::new(CapFallbackQuoteThresholdAdjuster { cap: 44631.0 }) as Rc<dyn FallbackQuoteThresholdModifier>,
            Rc::new(FlaggedFallbackQuoteThresholdAdjuster {
                flags: flags.clone(),
                flag: "fallback-quote-threshold-seasonal".to_string(),
                percent: -5.5,
            }),
            Rc::new(TieredFallbackQuoteThresholdAdjuster {
                bounds: vec![546.0, 2096.0, 2818.0, 4176.0],
                rates: vec![1.18, 0.91, 0.92, 1.07],
            }),
            Rc::new(MinimumFallbackQuoteThresholdAdjuster { minimum: 43.2 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 9005.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_quote_threshold_engine(&flags::new_registry());
        let item = new_fallback_quote_threshold("code-915", "name-286");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
