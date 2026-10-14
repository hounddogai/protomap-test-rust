use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched segment forecast calculation.
pub trait BatchedSegmentForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedSegmentForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedSegmentForecastAdjuster {
    pub cap: f64,
}

impl BatchedSegmentForecastAdjuster for CapBatchedSegmentForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedSegmentForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedSegmentForecastAdjuster {
    pub step: f64,
}

impl BatchedSegmentForecastAdjuster for RoundingBatchedSegmentForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedSegmentForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelBatchedSegmentForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedSegmentForecastAdjuster for LabelBatchedSegmentForecastAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &BatchedSegmentForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedSegmentForecastAdjuster {
    pub percent: f64,
}

impl BatchedSegmentForecastAdjuster for PercentBatchedSegmentForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedSegmentForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched segment forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedSegmentForecastEngine {
    pub steps: Vec<Rc<dyn BatchedSegmentForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedSegmentForecastEngine {
    pub fn evaluate(&mut self, item: &BatchedSegmentForecast) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedSegmentForecastAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedSegmentForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_segment_forecast_engine() -> BatchedSegmentForecastEngine {
    BatchedSegmentForecastEngine {
        steps: vec![
            Rc::new(CapBatchedSegmentForecastAdjuster { cap: 31645.0 }) as Rc<dyn BatchedSegmentForecastAdjuster>,
            Rc::new(RoundingBatchedSegmentForecastAdjuster { step: 0.5 }),
            Rc::new(LabelBatchedSegmentForecastAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.79),
                    ("retail".to_string(), 1.27),
                    ("east".to_string(), 1.33),
                ]),
            }),
            Rc::new(PercentBatchedSegmentForecastAdjuster { percent: -23.1 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 4953.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_segment_forecast_engine();
        let item = new_batched_segment_forecast("code-574", "label-711");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
