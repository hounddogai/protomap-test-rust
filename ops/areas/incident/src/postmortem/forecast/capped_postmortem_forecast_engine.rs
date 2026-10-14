use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the capped postmortem forecast calculation.
pub trait CappedPostmortemForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedPostmortemForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedPostmortemForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedPostmortemForecastAdjuster for TieredCappedPostmortemForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedPostmortemForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemForecastRevisionOffsetAdjuster {
    pub weight: f64,
}

impl CappedPostmortemForecastAdjuster for CappedPostmortemForecastRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &CappedPostmortemForecast, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedPostmortemForecastAdjuster {
    pub cap: f64,
}

impl CappedPostmortemForecastAdjuster for CapCappedPostmortemForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedPostmortemForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the capped postmortem forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedPostmortemForecastEngine {
    pub steps: Vec<Rc<dyn CappedPostmortemForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedPostmortemForecastEngine {
    pub fn run(&mut self, item: &CappedPostmortemForecast) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedPostmortemForecastAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedPostmortemForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_postmortem_forecast_engine() -> CappedPostmortemForecastEngine {
    CappedPostmortemForecastEngine {
        steps: vec![
            Rc::new(TieredCappedPostmortemForecastAdjuster {
                bounds: vec![2031.0, 7734.0, 9447.0],
                rates: vec![1.06, 1.02, 0.94],
            }) as Rc<dyn CappedPostmortemForecastAdjuster>,
            Rc::new(CappedPostmortemForecastRevisionOffsetAdjuster { weight: 0.43 }),
            Rc::new(CapCappedPostmortemForecastAdjuster { cap: 18926.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 33309.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_postmortem_forecast_engine();
        let item = new_capped_postmortem_forecast("code-448", "label-727");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
