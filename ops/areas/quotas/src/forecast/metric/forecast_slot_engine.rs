use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the forecast slot calculation.
pub trait ForecastSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSlotRatioOffsetAdjuster {
    pub weight: f64,
}

impl ForecastSlotStep for ForecastSlotRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &ForecastSlot, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastSlotAdjuster {
    pub step: f64,
}

impl ForecastSlotStep for RoundingForecastSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryForecastSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastSlotStep for CountryForecastSlotAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ForecastSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentForecastSlotAdjuster {
    pub percent: f64,
}

impl ForecastSlotStep for PercentForecastSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ForecastSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastSlotStep for TieredForecastSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastSlotAdjuster {
    pub minimum: f64,
}

impl ForecastSlotStep for MinimumForecastSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the forecast slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastSlotEngine {
    pub steps: Vec<Rc<dyn ForecastSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastSlotEngine {
    pub fn compute(&mut self, item: &ForecastSlot) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ForecastSlotStep> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[ForecastSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_forecast_slot_engine() -> ForecastSlotEngine {
    ForecastSlotEngine {
        steps: vec![
            Rc::new(ForecastSlotRatioOffsetAdjuster { weight: 1.54 }) as Rc<dyn ForecastSlotStep>,
            Rc::new(RoundingForecastSlotAdjuster { step: 0.25 }),
            Rc::new(CountryForecastSlotAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.99),
                    ("south".to_string(), 0.74),
                    ("north".to_string(), 0.78),
                ]),
            }),
            Rc::new(PercentForecastSlotAdjuster { percent: -3.5 }),
            Rc::new(TieredForecastSlotAdjuster {
                bounds: vec![3790.0, 6550.0, 6823.0],
                rates: vec![0.82, 1.17, 0.87],
            }),
            Rc::new(MinimumForecastSlotAdjuster { minimum: 39.2 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 40757.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_slot_engine();
        let item = new_forecast_slot("code-79", "title-537");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
