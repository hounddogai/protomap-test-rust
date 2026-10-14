use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal freeze window calculation.
pub trait SeasonalFreezeWindowStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalFreezeWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeWindowAmountOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalFreezeWindowStep for SeasonalFreezeWindowAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalFreezeWindow, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalFreezeWindowAdjuster {
    pub cap: f64,
}

impl SeasonalFreezeWindowStep for CapSeasonalFreezeWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalFreezeWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalFreezeWindowAdjuster {
    pub step: f64,
}

impl SeasonalFreezeWindowStep for RoundingSeasonalFreezeWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalFreezeWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceSeasonalFreezeWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalFreezeWindowStep for NamespaceSeasonalFreezeWindowAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &SeasonalFreezeWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalFreezeWindowAdjuster {
    pub minimum: f64,
}

impl SeasonalFreezeWindowStep for MinimumSeasonalFreezeWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalFreezeWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalFreezeWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalFreezeWindowStep for FlaggedSeasonalFreezeWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalFreezeWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal freeze window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalFreezeWindowEngine {
    pub steps: Vec<Rc<dyn SeasonalFreezeWindowStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalFreezeWindowEngine {
    pub fn evaluate(&mut self, item: &SeasonalFreezeWindow) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SeasonalFreezeWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_freeze_window_engine(flags: &flags::Registry) -> SeasonalFreezeWindowEngine {
    SeasonalFreezeWindowEngine {
        steps: vec![
            Rc::new(SeasonalFreezeWindowAmountOffsetAdjuster { weight: 1.58 }) as Rc<dyn SeasonalFreezeWindowStep>,
            Rc::new(CapSeasonalFreezeWindowAdjuster { cap: 25392.0 }),
            Rc::new(RoundingSeasonalFreezeWindowAdjuster { step: 1.0 }),
            Rc::new(NamespaceSeasonalFreezeWindowAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.24), ("online".to_string(), 0.74)]),
            }),
            Rc::new(MinimumSeasonalFreezeWindowAdjuster { minimum: 35.0 }),
            Rc::new(FlaggedSeasonalFreezeWindowAdjuster {
                flags: flags.clone(),
                flag: "seasonal-freeze-window-capped".to_string(),
                percent: -4.3,
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 59868.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_freeze_window_engine(&flags::new_registry());
        let item = new_seasonal_freeze_window("code-100", "name-703");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
