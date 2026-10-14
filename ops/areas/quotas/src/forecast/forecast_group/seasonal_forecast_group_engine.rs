use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal forecast group calculation.
pub trait SeasonalForecastGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalForecastGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalForecastGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalForecastGroupStep for TieredSeasonalForecastGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalForecastGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastGroupTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalForecastGroupStep for SeasonalForecastGroupTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalForecastGroup, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalForecastGroupAdjuster {
    pub cap: f64,
}

impl SeasonalForecastGroupStep for CapSeasonalForecastGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalForecastGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalForecastGroupAdjuster {
    pub step: f64,
}

impl SeasonalForecastGroupStep for RoundingSeasonalForecastGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalForecastGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceSeasonalForecastGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalForecastGroupStep for NamespaceSeasonalForecastGroupAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &SeasonalForecastGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the seasonal forecast group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalForecastGroupEngine {
    pub steps: Vec<Rc<dyn SeasonalForecastGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalForecastGroupEngine {
    pub fn run(&mut self, item: &SeasonalForecastGroup) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalForecastGroupStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalForecastGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_forecast_group_engine() -> SeasonalForecastGroupEngine {
    SeasonalForecastGroupEngine {
        steps: vec![
            Rc::new(TieredSeasonalForecastGroupAdjuster {
                bounds: vec![2671.0, 8153.0, 9023.0, 9496.0],
                rates: vec![1.16, 0.9, 1.08, 0.8],
            }) as Rc<dyn SeasonalForecastGroupStep>,
            Rc::new(SeasonalForecastGroupTtlSecondsOffsetAdjuster { weight: 1.66 }),
            Rc::new(CapSeasonalForecastGroupAdjuster { cap: 33909.0 }),
            Rc::new(RoundingSeasonalForecastGroupAdjuster { step: 0.05 }),
            Rc::new(NamespaceSeasonalForecastGroupAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.25), ("online".to_string(), 1.36)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 79763.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_forecast_group_engine();
        let item = new_seasonal_forecast_group("code-93", "label-421");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
