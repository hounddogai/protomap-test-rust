use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the data export segment calculation.
pub trait DataExportSegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportSegmentAdjuster {
    pub minimum: f64,
}

impl DataExportSegmentAdjuster for MinimumDataExportSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryDataExportSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportSegmentAdjuster for CategoryDataExportSegmentAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &DataExportSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportSegmentAdjuster for FlaggedDataExportSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the data export segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportSegmentEngine {
    pub steps: Vec<Rc<dyn DataExportSegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportSegmentEngine {
    pub fn evaluate(&mut self, item: &DataExportSegment) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DataExportSegmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DataExportSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_data_export_segment_engine(flags: &flags::Registry) -> DataExportSegmentEngine {
    DataExportSegmentEngine {
        steps: vec![
            Rc::new(MinimumDataExportSegmentAdjuster { minimum: 28.1 }) as Rc<dyn DataExportSegmentAdjuster>,
            Rc::new(CategoryDataExportSegmentAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.35)]),
            }),
            Rc::new(FlaggedDataExportSegmentAdjuster {
                flags: flags.clone(),
                flag: "data-export-segment-legacy".to_string(),
                percent: -5.7,
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 79630.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_segment_engine(&flags::new_registry());
        let item = new_data_export_segment("code-327", "label-465");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
