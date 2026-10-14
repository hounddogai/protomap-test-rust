use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy allowlist baseline calculation.
pub trait LegacyAllowlistBaselineModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyAllowlistBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyAllowlistBaselineAdjuster {
    pub percent: f64,
}

impl LegacyAllowlistBaselineModifier for PercentLegacyAllowlistBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyAllowlistBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyAllowlistBaselineAdjuster {
    pub cap: f64,
}

impl LegacyAllowlistBaselineModifier for CapLegacyAllowlistBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyAllowlistBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyAllowlistBaselineAdjuster {
    pub step: f64,
}

impl LegacyAllowlistBaselineModifier for RoundingLegacyAllowlistBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyAllowlistBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyAllowlistBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyAllowlistBaselineModifier for TieredLegacyAllowlistBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyAllowlistBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyAllowlistBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyAllowlistBaselineModifier for FlaggedLegacyAllowlistBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyAllowlistBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy allowlist baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyAllowlistBaselineEngine {
    pub steps: Vec<Rc<dyn LegacyAllowlistBaselineModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyAllowlistBaselineEngine {
    pub fn price(&mut self, item: &LegacyAllowlistBaseline) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[LegacyAllowlistBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_allowlist_baseline_engine(flags: &flags::Registry) -> LegacyAllowlistBaselineEngine {
    LegacyAllowlistBaselineEngine {
        steps: vec![
            Rc::new(PercentLegacyAllowlistBaselineAdjuster { percent: -13.5 })
                as Rc<dyn LegacyAllowlistBaselineModifier>,
            Rc::new(CapLegacyAllowlistBaselineAdjuster { cap: 37003.0 }),
            Rc::new(RoundingLegacyAllowlistBaselineAdjuster { step: 0.5 }),
            Rc::new(TieredLegacyAllowlistBaselineAdjuster {
                bounds: vec![3855.0, 3875.0, 3945.0],
                rates: vec![0.81, 0.83, 1.11],
            }),
            Rc::new(FlaggedLegacyAllowlistBaselineAdjuster {
                flags: flags.clone(),
                flag: "legacy-allowlist-baseline-capped".to_string(),
                percent: -3.4,
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 50547.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_allowlist_baseline_engine(&flags::new_registry());
        let item = new_legacy_allowlist_baseline("code-813", "label-222");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
