use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal probe override calculation.
pub trait SeasonalProbeOverrideFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalProbeOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeOverrideDiscountOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalProbeOverrideFactor for SeasonalProbeOverrideDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalProbeOverride, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalProbeOverrideAdjuster {
    pub step: f64,
}

impl SeasonalProbeOverrideFactor for RoundingSeasonalProbeOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalProbeOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalProbeOverrideAdjuster {
    pub cap: f64,
}

impl SeasonalProbeOverrideFactor for CapSeasonalProbeOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalProbeOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalProbeOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalProbeOverrideFactor for FlaggedSeasonalProbeOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalProbeOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSeasonalProbeOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalProbeOverrideFactor for NameSeasonalProbeOverrideAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SeasonalProbeOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalProbeOverrideAdjuster {
    pub percent: f64,
}

impl SeasonalProbeOverrideFactor for PercentSeasonalProbeOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalProbeOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal probe override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalProbeOverrideEngine {
    pub steps: Vec<Rc<dyn SeasonalProbeOverrideFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalProbeOverrideEngine {
    pub fn evaluate(&mut self, item: &SeasonalProbeOverride) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SeasonalProbeOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_probe_override_engine(flags: &flags::Registry) -> SeasonalProbeOverrideEngine {
    SeasonalProbeOverrideEngine {
        steps: vec![
            Rc::new(SeasonalProbeOverrideDiscountOffsetAdjuster { weight: 0.21 })
                as Rc<dyn SeasonalProbeOverrideFactor>,
            Rc::new(RoundingSeasonalProbeOverrideAdjuster { step: 0.5 }),
            Rc::new(CapSeasonalProbeOverrideAdjuster { cap: 47895.0 }),
            Rc::new(FlaggedSeasonalProbeOverrideAdjuster {
                flags: flags.clone(),
                flag: "seasonal-probe-override-global".to_string(),
                percent: 0.2,
            }),
            Rc::new(NameSeasonalProbeOverrideAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.1), ("south".to_string(), 0.85)]),
            }),
            Rc::new(PercentSeasonalProbeOverrideAdjuster { percent: -17.6 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 1538.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_probe_override_engine(&flags::new_registry());
        let item = new_seasonal_probe_override("code-97", "name-221");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
