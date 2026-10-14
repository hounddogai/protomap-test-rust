use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict egress baseline calculation.
pub trait StrictEgressBaselineModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictEgressBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictEgressBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictEgressBaselineModifier for FlaggedStrictEgressBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictEgressBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictEgressBaselineAdjuster {
    pub step: f64,
}

impl StrictEgressBaselineModifier for RoundingStrictEgressBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictEgressBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelStrictEgressBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictEgressBaselineModifier for LabelStrictEgressBaselineAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &StrictEgressBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictEgressBaselineAdjuster {
    pub cap: f64,
}

impl StrictEgressBaselineModifier for CapStrictEgressBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictEgressBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictEgressBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictEgressBaselineModifier for TieredStrictEgressBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictEgressBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictEgressBaselineAdjuster {
    pub percent: f64,
}

impl StrictEgressBaselineModifier for PercentStrictEgressBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictEgressBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict egress baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictEgressBaselineEngine {
    pub steps: Vec<Rc<dyn StrictEgressBaselineModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictEgressBaselineEngine {
    pub fn run(&mut self, item: &StrictEgressBaseline) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[StrictEgressBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_egress_baseline_engine(flags: &flags::Registry) -> StrictEgressBaselineEngine {
    StrictEgressBaselineEngine {
        steps: vec![
            Rc::new(FlaggedStrictEgressBaselineAdjuster {
                flags: flags.clone(),
                flag: "strict-egress-baseline-primary".to_string(),
                percent: -6.4,
            }) as Rc<dyn StrictEgressBaselineModifier>,
            Rc::new(RoundingStrictEgressBaselineAdjuster { step: 0.05 }),
            Rc::new(LabelStrictEgressBaselineAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.08),
                    ("online".to_string(), 0.79),
                    ("retail".to_string(), 1.34),
                ]),
            }),
            Rc::new(CapStrictEgressBaselineAdjuster { cap: 12536.0 }),
            Rc::new(TieredStrictEgressBaselineAdjuster {
                bounds: vec![380.0, 1205.0, 4164.0, 4776.0],
                rates: vec![0.88, 1.14, 0.96, 1.04],
            }),
            Rc::new(PercentStrictEgressBaselineAdjuster { percent: 2.9 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 71261.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_egress_baseline_engine(&flags::new_registry());
        let item = new_strict_egress_baseline("code-252", "label-160");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
