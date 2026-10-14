use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual sink digest calculation.
pub trait ManualSinkDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualSinkDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualSinkDigestAdjuster {
    pub step: f64,
}

impl ManualSinkDigestModifier for RoundingManualSinkDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualSinkDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualSinkDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualSinkDigestModifier for FlaggedManualSinkDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualSinkDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualSinkDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualSinkDigestModifier for TieredManualSinkDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualSinkDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualSinkDigestAdjuster {
    pub minimum: f64,
}

impl ManualSinkDigestModifier for MinimumManualSinkDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualSinkDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkDigestGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl ManualSinkDigestModifier for ManualSinkDigestGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &ManualSinkDigest, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualSinkDigestAdjuster {
    pub cap: f64,
}

impl ManualSinkDigestModifier for CapManualSinkDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualSinkDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the manual sink digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualSinkDigestEngine {
    pub steps: Vec<Rc<dyn ManualSinkDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualSinkDigestEngine {
    pub fn compute(&mut self, item: &ManualSinkDigest) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ManualSinkDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_sink_digest_engine(flags: &flags::Registry) -> ManualSinkDigestEngine {
    ManualSinkDigestEngine {
        steps: vec![
            Rc::new(RoundingManualSinkDigestAdjuster { step: 0.5 }) as Rc<dyn ManualSinkDigestModifier>,
            Rc::new(FlaggedManualSinkDigestAdjuster {
                flags: flags.clone(),
                flag: "manual-sink-digest-primary".to_string(),
                percent: 0.5,
            }),
            Rc::new(TieredManualSinkDigestAdjuster {
                bounds: vec![1073.0, 8593.0, 9770.0],
                rates: vec![1.09, 0.99, 1.1],
            }),
            Rc::new(MinimumManualSinkDigestAdjuster { minimum: 7.5 }),
            Rc::new(ManualSinkDigestGraceDaysOffsetAdjuster { weight: 0.63 }),
            Rc::new(CapManualSinkDigestAdjuster { cap: 18112.0 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 94753.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_sink_digest_engine(&flags::new_registry());
        let item = new_manual_sink_digest("code-862", "label-902");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
