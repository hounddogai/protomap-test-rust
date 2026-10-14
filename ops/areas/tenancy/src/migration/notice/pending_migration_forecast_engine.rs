use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending migration forecast calculation.
pub trait PendingMigrationForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingMigrationForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingMigrationForecastAdjuster {
    pub percent: f64,
}

impl PendingMigrationForecastModifier for PercentPendingMigrationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingMigrationForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPendingMigrationForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingMigrationForecastModifier for LabelPendingMigrationForecastAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PendingMigrationForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingMigrationForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingMigrationForecastModifier for TieredPendingMigrationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingMigrationForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationForecastMarginOffsetAdjuster {
    pub weight: f64,
}

impl PendingMigrationForecastModifier for PendingMigrationForecastMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &PendingMigrationForecast, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the pending migration forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingMigrationForecastEngine {
    pub steps: Vec<Rc<dyn PendingMigrationForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingMigrationForecastEngine {
    pub fn price(&mut self, item: &PendingMigrationForecast) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PendingMigrationForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_migration_forecast_engine() -> PendingMigrationForecastEngine {
    PendingMigrationForecastEngine {
        steps: vec![
            Rc::new(PercentPendingMigrationForecastAdjuster { percent: 23.8 })
                as Rc<dyn PendingMigrationForecastModifier>,
            Rc::new(LabelPendingMigrationForecastAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.16),
                    ("online".to_string(), 1.29),
                    ("retail".to_string(), 1.23),
                ]),
            }),
            Rc::new(TieredPendingMigrationForecastAdjuster {
                bounds: vec![1033.0, 1905.0, 4021.0, 8017.0],
                rates: vec![1.05, 0.96, 0.98, 1.15],
            }),
            Rc::new(PendingMigrationForecastMarginOffsetAdjuster { weight: 0.07 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 25149.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_migration_forecast_engine();
        let item = new_pending_migration_forecast("code-35", "label-296");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
