use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict forecast cycle calculation.
pub trait StrictForecastCycleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictForecastCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictForecastCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictForecastCycleAdjuster for TieredStrictForecastCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictForecastCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictForecastCycleAdjuster {
    pub step: f64,
}

impl StrictForecastCycleAdjuster for RoundingStrictForecastCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictForecastCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictForecastCycleAdjuster {
    pub percent: f64,
}

impl StrictForecastCycleAdjuster for PercentStrictForecastCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictForecastCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelStrictForecastCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictForecastCycleAdjuster for LabelStrictForecastCycleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &StrictForecastCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastCycleQuantityOffsetAdjuster {
    pub weight: f64,
}

impl StrictForecastCycleAdjuster for StrictForecastCycleQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &StrictForecastCycle, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictForecastCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictForecastCycleAdjuster for FlaggedStrictForecastCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictForecastCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict forecast cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictForecastCycleEngine {
    pub steps: Vec<Rc<dyn StrictForecastCycleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictForecastCycleEngine {
    pub fn price(&mut self, item: &StrictForecastCycle) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[StrictForecastCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_forecast_cycle_engine(flags: &flags::Registry) -> StrictForecastCycleEngine {
    StrictForecastCycleEngine {
        steps: vec![
            Rc::new(TieredStrictForecastCycleAdjuster {
                bounds: vec![2930.0, 3766.0, 5251.0, 7620.0],
                rates: vec![1.16, 0.94, 0.91, 1.06],
            }) as Rc<dyn StrictForecastCycleAdjuster>,
            Rc::new(RoundingStrictForecastCycleAdjuster { step: 1.0 }),
            Rc::new(PercentStrictForecastCycleAdjuster { percent: 17.1 }),
            Rc::new(LabelStrictForecastCycleAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.33),
                    ("retail".to_string(), 1.03),
                    ("central".to_string(), 1.27),
                ]),
            }),
            Rc::new(StrictForecastCycleQuantityOffsetAdjuster { weight: 0.74 }),
            Rc::new(FlaggedStrictForecastCycleAdjuster {
                flags: flags.clone(),
                flag: "strict-forecast-cycle-pending".to_string(),
                percent: -1.9,
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 38352.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_forecast_cycle_engine(&flags::new_registry());
        let item = new_strict_forecast_cycle("code-661", "label-941");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
