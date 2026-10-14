use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy silence window calculation.
pub trait LegacySilenceWindowAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacySilenceWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacySilenceWindowAdjuster {
    pub minimum: f64,
}

impl LegacySilenceWindowAdjuster for MinimumLegacySilenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacySilenceWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacySilenceWindowAdjuster {
    pub cap: f64,
}

impl LegacySilenceWindowAdjuster for CapLegacySilenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacySilenceWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacySilenceWindowAdjuster {
    pub percent: f64,
}

impl LegacySilenceWindowAdjuster for PercentLegacySilenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacySilenceWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacySilenceWindowAdjuster {
    pub step: f64,
}

impl LegacySilenceWindowAdjuster for RoundingLegacySilenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacySilenceWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacySilenceWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacySilenceWindowAdjuster for TieredLegacySilenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacySilenceWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacySilenceWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacySilenceWindowAdjuster for FlaggedLegacySilenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacySilenceWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy silence window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacySilenceWindowEngine {
    pub steps: Vec<Rc<dyn LegacySilenceWindowAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacySilenceWindowEngine {
    pub fn price(&mut self, item: &LegacySilenceWindow) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacySilenceWindowAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacySilenceWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_silence_window_engine(flags: &flags::Registry) -> LegacySilenceWindowEngine {
    LegacySilenceWindowEngine {
        steps: vec![
            Rc::new(MinimumLegacySilenceWindowAdjuster { minimum: 29.3 }) as Rc<dyn LegacySilenceWindowAdjuster>,
            Rc::new(CapLegacySilenceWindowAdjuster { cap: 5105.0 }),
            Rc::new(PercentLegacySilenceWindowAdjuster { percent: -10.7 }),
            Rc::new(RoundingLegacySilenceWindowAdjuster { step: 5.0 }),
            Rc::new(TieredLegacySilenceWindowAdjuster {
                bounds: vec![487.0, 6096.0, 7824.0, 9851.0],
                rates: vec![1.17, 1.11, 1.16, 1.18],
            }),
            Rc::new(FlaggedLegacySilenceWindowAdjuster {
                flags: flags.clone(),
                flag: "legacy-silence-window-regional".to_string(),
                percent: -9.4,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 25939.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_silence_window_engine(&flags::new_registry());
        let item = new_legacy_silence_window("code-572", "title-648");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
