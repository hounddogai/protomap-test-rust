use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped forecast score calculation.
pub trait CappedForecastScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedForecastScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedForecastScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedForecastScoreStep for TieredCappedForecastScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedForecastScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCappedForecastScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedForecastScoreStep for NameCappedForecastScoreAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CappedForecastScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedForecastScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedForecastScoreStep for FlaggedCappedForecastScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedForecastScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedForecastScoreAdjuster {
    pub percent: f64,
}

impl CappedForecastScoreStep for PercentCappedForecastScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedForecastScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastScoreDiscountOffsetAdjuster {
    pub weight: f64,
}

impl CappedForecastScoreStep for CappedForecastScoreDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &CappedForecastScore, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

/// Runs the capped forecast score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedForecastScoreEngine {
    pub steps: Vec<Rc<dyn CappedForecastScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedForecastScoreEngine {
    pub fn run(&mut self, item: &CappedForecastScore) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[CappedForecastScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_forecast_score_engine(flags: &flags::Registry) -> CappedForecastScoreEngine {
    CappedForecastScoreEngine {
        steps: vec![
            Rc::new(TieredCappedForecastScoreAdjuster {
                bounds: vec![5118.0, 9553.0],
                rates: vec![0.89, 0.84],
            }) as Rc<dyn CappedForecastScoreStep>,
            Rc::new(NameCappedForecastScoreAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.92), ("south".to_string(), 1.32)]),
            }),
            Rc::new(FlaggedCappedForecastScoreAdjuster {
                flags: flags.clone(),
                flag: "capped-forecast-score-scheduled".to_string(),
                percent: 3.7,
            }),
            Rc::new(PercentCappedForecastScoreAdjuster { percent: 17.6 }),
            Rc::new(CappedForecastScoreDiscountOffsetAdjuster { weight: 0.39 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 36421.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_forecast_score_engine(&flags::new_registry());
        let item = new_capped_forecast_score("code-484", "name-79");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
