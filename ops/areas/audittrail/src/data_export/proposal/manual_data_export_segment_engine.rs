use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the manual data export segment calculation.
pub trait ManualDataExportSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualDataExportSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionManualDataExportSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualDataExportSegmentStep for RegionManualDataExportSegmentAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ManualDataExportSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualDataExportSegmentAdjuster {
    pub percent: f64,
}

impl ManualDataExportSegmentStep for PercentManualDataExportSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualDataExportSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualDataExportSegmentAdjuster {
    pub minimum: f64,
}

impl ManualDataExportSegmentStep for MinimumManualDataExportSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualDataExportSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual data export segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualDataExportSegmentEngine {
    pub steps: Vec<Rc<dyn ManualDataExportSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualDataExportSegmentEngine {
    pub fn run(&mut self, item: &ManualDataExportSegment) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualDataExportSegmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualDataExportSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_data_export_segment_engine() -> ManualDataExportSegmentEngine {
    ManualDataExportSegmentEngine {
        steps: vec![
            Rc::new(RegionManualDataExportSegmentAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.93), ("retail".to_string(), 1.09)]),
            }) as Rc<dyn ManualDataExportSegmentStep>,
            Rc::new(PercentManualDataExportSegmentAdjuster { percent: 22.9 }),
            Rc::new(MinimumManualDataExportSegmentAdjuster { minimum: 26.9 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 64455.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_data_export_segment_engine();
        let item = new_manual_data_export_segment("code-534", "title-223");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
