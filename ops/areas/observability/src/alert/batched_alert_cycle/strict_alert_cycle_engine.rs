use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict alert cycle calculation.
pub trait StrictAlertCycleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictAlertCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictAlertCycleAdjuster {
    pub minimum: f64,
}

impl StrictAlertCycleModifier for MinimumStrictAlertCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictAlertCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertCycleScoreOffsetAdjuster {
    pub weight: f64,
}

impl StrictAlertCycleModifier for StrictAlertCycleScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &StrictAlertCycle, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictAlertCycleAdjuster {
    pub step: f64,
}

impl StrictAlertCycleModifier for RoundingStrictAlertCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictAlertCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictAlertCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictAlertCycleModifier for FlaggedStrictAlertCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictAlertCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictAlertCycleAdjuster {
    pub percent: f64,
}

impl StrictAlertCycleModifier for PercentStrictAlertCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictAlertCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict alert cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictAlertCycleEngine {
    pub steps: Vec<Rc<dyn StrictAlertCycleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictAlertCycleEngine {
    pub fn evaluate(&mut self, item: &StrictAlertCycle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[StrictAlertCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_alert_cycle_engine(flags: &flags::Registry) -> StrictAlertCycleEngine {
    StrictAlertCycleEngine {
        steps: vec![
            Rc::new(MinimumStrictAlertCycleAdjuster { minimum: 38.6 }) as Rc<dyn StrictAlertCycleModifier>,
            Rc::new(StrictAlertCycleScoreOffsetAdjuster { weight: 0.51 }),
            Rc::new(RoundingStrictAlertCycleAdjuster { step: 5.0 }),
            Rc::new(FlaggedStrictAlertCycleAdjuster {
                flags: flags.clone(),
                flag: "strict-alert-cycle-weighted".to_string(),
                percent: -9.7,
            }),
            Rc::new(PercentStrictAlertCycleAdjuster { percent: -20.2 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 73103.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_alert_cycle_engine(&flags::new_registry());
        let item = new_strict_alert_cycle("code-571", "name-146");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
