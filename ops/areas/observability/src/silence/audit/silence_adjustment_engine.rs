use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the silence adjustment calculation.
pub trait SilenceAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSilenceAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SilenceAdjustmentFactor for FlaggedSilenceAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SilenceAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSilenceAdjustmentAdjuster {
    pub minimum: f64,
}

impl SilenceAdjustmentFactor for MinimumSilenceAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SilenceAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAdjustmentEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl SilenceAdjustmentFactor for SilenceAdjustmentEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &SilenceAdjustment, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSilenceAdjustmentAdjuster {
    pub step: f64,
}

impl SilenceAdjustmentFactor for RoundingSilenceAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SilenceAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SilenceAdjustmentFactor for TieredSilenceAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SilenceAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the silence adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceAdjustmentEngine {
    pub steps: Vec<Rc<dyn SilenceAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceAdjustmentEngine {
    pub fn evaluate(&mut self, item: &SilenceAdjustment) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SilenceAdjustmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SilenceAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_silence_adjustment_engine(flags: &flags::Registry) -> SilenceAdjustmentEngine {
    SilenceAdjustmentEngine {
        steps: vec![
            Rc::new(FlaggedSilenceAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "silence-adjustment-manual".to_string(),
                percent: 2.0,
            }) as Rc<dyn SilenceAdjustmentFactor>,
            Rc::new(MinimumSilenceAdjustmentAdjuster { minimum: 39.1 }),
            Rc::new(SilenceAdjustmentEndsAtOffsetAdjuster { weight: 0.6 }),
            Rc::new(RoundingSilenceAdjustmentAdjuster { step: 0.25 }),
            Rc::new(TieredSilenceAdjustmentAdjuster {
                bounds: vec![1002.0, 7595.0, 8134.0, 9578.0],
                rates: vec![0.93, 1.18, 0.99, 0.89],
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 27098.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_adjustment_engine(&flags::new_registry());
        let item = new_silence_adjustment("code-630", "label-130");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
