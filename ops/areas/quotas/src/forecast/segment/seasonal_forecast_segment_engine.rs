use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal forecast segment calculation.
pub trait SeasonalForecastSegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalForecastSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalForecastSegmentAdjuster {
    pub cap: f64,
}

impl SeasonalForecastSegmentAdjuster for CapSeasonalForecastSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalForecastSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalForecastSegmentAdjuster {
    pub percent: f64,
}

impl SeasonalForecastSegmentAdjuster for PercentSeasonalForecastSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalForecastSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalForecastSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalForecastSegmentAdjuster for TieredSeasonalForecastSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalForecastSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalForecastSegmentAdjuster {
    pub step: f64,
}

impl SeasonalForecastSegmentAdjuster for RoundingSeasonalForecastSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalForecastSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastSegmentThresholdOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalForecastSegmentAdjuster for SeasonalForecastSegmentThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalForecastSegment, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalForecastSegmentAdjuster {
    pub minimum: f64,
}

impl SeasonalForecastSegmentAdjuster for MinimumSeasonalForecastSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalForecastSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the seasonal forecast segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalForecastSegmentEngine {
    pub steps: Vec<Rc<dyn SeasonalForecastSegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalForecastSegmentEngine {
    pub fn compute(&mut self, item: &SeasonalForecastSegment) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalForecastSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_forecast_segment_engine() -> SeasonalForecastSegmentEngine {
    SeasonalForecastSegmentEngine {
        steps: vec![
            Rc::new(CapSeasonalForecastSegmentAdjuster { cap: 28406.0 }) as Rc<dyn SeasonalForecastSegmentAdjuster>,
            Rc::new(PercentSeasonalForecastSegmentAdjuster { percent: -7.1 }),
            Rc::new(TieredSeasonalForecastSegmentAdjuster {
                bounds: vec![4792.0, 5073.0],
                rates: vec![0.93, 1.02],
            }),
            Rc::new(RoundingSeasonalForecastSegmentAdjuster { step: 0.5 }),
            Rc::new(SeasonalForecastSegmentThresholdOffsetAdjuster { weight: 1.43 }),
            Rc::new(MinimumSeasonalForecastSegmentAdjuster { minimum: 33.7 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 35786.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_forecast_segment_engine();
        let item = new_seasonal_forecast_segment("code-283", "label-532");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
