use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy rollback baseline calculation.
pub trait LegacyRollbackBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyRollbackBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyRollbackBaselineAdjuster {
    pub cap: f64,
}

impl LegacyRollbackBaselineStep for CapLegacyRollbackBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyRollbackBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyRollbackBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyRollbackBaselineStep for TieredLegacyRollbackBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyRollbackBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryLegacyRollbackBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyRollbackBaselineStep for CountryLegacyRollbackBaselineAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &LegacyRollbackBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyRollbackBaselineAdjuster {
    pub percent: f64,
}

impl LegacyRollbackBaselineStep for PercentLegacyRollbackBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyRollbackBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyRollbackBaselineAdjuster {
    pub step: f64,
}

impl LegacyRollbackBaselineStep for RoundingLegacyRollbackBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyRollbackBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyRollbackBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyRollbackBaselineStep for FlaggedLegacyRollbackBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyRollbackBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy rollback baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyRollbackBaselineEngine {
    pub steps: Vec<Rc<dyn LegacyRollbackBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyRollbackBaselineEngine {
    pub fn compute(&mut self, item: &LegacyRollbackBaseline) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LegacyRollbackBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_rollback_baseline_engine(flags: &flags::Registry) -> LegacyRollbackBaselineEngine {
    LegacyRollbackBaselineEngine {
        steps: vec![
            Rc::new(CapLegacyRollbackBaselineAdjuster { cap: 614.0 }) as Rc<dyn LegacyRollbackBaselineStep>,
            Rc::new(TieredLegacyRollbackBaselineAdjuster {
                bounds: vec![1101.0, 4519.0, 6310.0, 7903.0],
                rates: vec![0.99, 0.86, 0.96, 0.87],
            }),
            Rc::new(CountryLegacyRollbackBaselineAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.32)]),
            }),
            Rc::new(PercentLegacyRollbackBaselineAdjuster { percent: 1.7 }),
            Rc::new(RoundingLegacyRollbackBaselineAdjuster { step: 0.25 }),
            Rc::new(FlaggedLegacyRollbackBaselineAdjuster {
                flags: flags.clone(),
                flag: "legacy-rollback-baseline-pending".to_string(),
                percent: 2.6,
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 74275.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_rollback_baseline_engine(&flags::new_registry());
        let item = new_legacy_rollback_baseline("code-732", "title-18");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
