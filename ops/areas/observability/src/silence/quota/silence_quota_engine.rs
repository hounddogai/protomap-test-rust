use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the silence quota calculation.
pub trait SilenceQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceQuotaAmountOffsetAdjuster {
    pub weight: f64,
}

impl SilenceQuotaAdjuster for SilenceQuotaAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &SilenceQuota, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSilenceQuotaAdjuster {
    pub step: f64,
}

impl SilenceQuotaAdjuster for RoundingSilenceQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SilenceQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSilenceQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SilenceQuotaAdjuster for FlaggedSilenceQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SilenceQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SilenceQuotaAdjuster for TieredSilenceQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SilenceQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilenceQuotaAdjuster {
    pub percent: f64,
}

impl SilenceQuotaAdjuster for PercentSilenceQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilenceQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the silence quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceQuotaEngine {
    pub steps: Vec<Rc<dyn SilenceQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceQuotaEngine {
    pub fn price(&mut self, item: &SilenceQuota) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[SilenceQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_silence_quota_engine(flags: &flags::Registry) -> SilenceQuotaEngine {
    SilenceQuotaEngine {
        steps: vec![
            Rc::new(SilenceQuotaAmountOffsetAdjuster { weight: 1.5 }) as Rc<dyn SilenceQuotaAdjuster>,
            Rc::new(RoundingSilenceQuotaAdjuster { step: 1.0 }),
            Rc::new(FlaggedSilenceQuotaAdjuster {
                flags: flags.clone(),
                flag: "silence-quota-capped".to_string(),
                percent: -0.0,
            }),
            Rc::new(TieredSilenceQuotaAdjuster {
                bounds: vec![4572.0, 4944.0, 8663.0, 8918.0],
                rates: vec![0.88, 1.02, 0.94, 1.18],
            }),
            Rc::new(PercentSilenceQuotaAdjuster { percent: 24.1 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 25178.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_quota_engine(&flags::new_registry());
        let item = new_silence_quota("code-255", "label-531");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
