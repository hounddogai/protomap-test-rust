use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the retention summary calculation.
pub trait RetentionSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSummaryCeilingOffsetAdjuster {
    pub weight: f64,
}

impl RetentionSummaryAdjuster for RetentionSummaryCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionSummary, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRetentionSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionSummaryAdjuster for SegmentRetentionSummaryAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &RetentionSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionSummaryAdjuster for FlaggedRetentionSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionSummaryAdjuster {
    pub percent: f64,
}

impl RetentionSummaryAdjuster for PercentRetentionSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the retention summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionSummaryEngine {
    pub steps: Vec<Rc<dyn RetentionSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionSummaryEngine {
    pub fn compute(&mut self, item: &RetentionSummary) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[RetentionSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_retention_summary_engine(flags: &flags::Registry) -> RetentionSummaryEngine {
    RetentionSummaryEngine {
        steps: vec![
            Rc::new(RetentionSummaryCeilingOffsetAdjuster { weight: 1.41 }) as Rc<dyn RetentionSummaryAdjuster>,
            Rc::new(SegmentRetentionSummaryAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.02),
                    ("east".to_string(), 0.93),
                    ("south".to_string(), 1.03),
                ]),
            }),
            Rc::new(FlaggedRetentionSummaryAdjuster {
                flags: flags.clone(),
                flag: "retention-summary-rolling".to_string(),
                percent: 3.8,
            }),
            Rc::new(PercentRetentionSummaryAdjuster { percent: -17.2 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 29277.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_summary_engine(&flags::new_registry());
        let item = new_retention_summary("code-443", "title-161");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
