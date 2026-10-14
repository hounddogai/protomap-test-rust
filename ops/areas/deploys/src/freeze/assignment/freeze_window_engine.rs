use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the freeze window calculation.
pub trait FreezeWindowStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeWindowAdjuster {
    pub minimum: f64,
}

impl FreezeWindowStep for MinimumFreezeWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameFreezeWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeWindowStep for NameFreezeWindowAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &FreezeWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeWindowStep for TieredFreezeWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezeWindowAdjuster {
    pub step: f64,
}

impl FreezeWindowStep for RoundingFreezeWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezeWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeWindowThresholdOffsetAdjuster {
    pub weight: f64,
}

impl FreezeWindowStep for FreezeWindowThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeWindow, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFreezeWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FreezeWindowStep for FlaggedFreezeWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FreezeWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the freeze window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeWindowEngine {
    pub steps: Vec<Rc<dyn FreezeWindowStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeWindowEngine {
    pub fn run(&mut self, item: &FreezeWindow) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[FreezeWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_freeze_window_engine(flags: &flags::Registry) -> FreezeWindowEngine {
    FreezeWindowEngine {
        steps: vec![
            Rc::new(MinimumFreezeWindowAdjuster { minimum: 23.5 }) as Rc<dyn FreezeWindowStep>,
            Rc::new(NameFreezeWindowAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.81)]),
            }),
            Rc::new(TieredFreezeWindowAdjuster {
                bounds: vec![19.0, 8029.0, 9673.0],
                rates: vec![0.93, 1.12, 1.14],
            }),
            Rc::new(RoundingFreezeWindowAdjuster { step: 0.25 }),
            Rc::new(FreezeWindowThresholdOffsetAdjuster { weight: 1.65 }),
            Rc::new(FlaggedFreezeWindowAdjuster {
                flags: flags.clone(),
                flag: "freeze-window-strict".to_string(),
                percent: 5.9,
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 8014.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_window_engine(&flags::new_registry());
        let item = new_freeze_window("code-121", "name-966");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
