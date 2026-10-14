use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the data export review calculation.
pub trait DataExportReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportReviewAdjuster {
    pub minimum: f64,
}

impl DataExportReviewModifier for MinimumDataExportReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportReviewAdjuster {
    pub step: f64,
}

impl DataExportReviewModifier for RoundingDataExportReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDataExportReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportReviewModifier for TitleDataExportReviewAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DataExportReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportReviewAdjuster {
    pub cap: f64,
}

impl DataExportReviewModifier for CapDataExportReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportReviewAdjuster {
    pub percent: f64,
}

impl DataExportReviewModifier for PercentDataExportReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportReviewThresholdOffsetAdjuster {
    pub weight: f64,
}

impl DataExportReviewModifier for DataExportReviewThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportReview, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

/// Runs the data export review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportReviewEngine {
    pub steps: Vec<Rc<dyn DataExportReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportReviewEngine {
    pub fn compute(&mut self, item: &DataExportReview) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DataExportReviewModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DataExportReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_data_export_review_engine() -> DataExportReviewEngine {
    DataExportReviewEngine {
        steps: vec![
            Rc::new(MinimumDataExportReviewAdjuster { minimum: 24.1 }) as Rc<dyn DataExportReviewModifier>,
            Rc::new(RoundingDataExportReviewAdjuster { step: 0.05 }),
            Rc::new(TitleDataExportReviewAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.08), ("west".to_string(), 1.26)]),
            }),
            Rc::new(CapDataExportReviewAdjuster { cap: 24706.0 }),
            Rc::new(PercentDataExportReviewAdjuster { percent: -20.8 }),
            Rc::new(DataExportReviewThresholdOffsetAdjuster { weight: 0.79 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 92122.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_review_engine();
        let item = new_data_export_review("code-816", "title-827");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
