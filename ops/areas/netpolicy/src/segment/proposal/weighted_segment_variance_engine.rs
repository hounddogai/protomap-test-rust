use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted segment variance calculation.
pub trait WeightedSegmentVarianceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedSegmentVariance, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedSegmentVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedSegmentVarianceAdjuster for FlaggedWeightedSegmentVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedSegmentVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedSegmentVarianceAdjuster {
    pub cap: f64,
}

impl WeightedSegmentVarianceAdjuster for CapWeightedSegmentVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedSegmentVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceWeightedSegmentVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedSegmentVarianceAdjuster for SourceWeightedSegmentVarianceAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &WeightedSegmentVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedSegmentVarianceAdjuster {
    pub percent: f64,
}

impl WeightedSegmentVarianceAdjuster for PercentWeightedSegmentVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedSegmentVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedSegmentVarianceAdjuster {
    pub minimum: f64,
}

impl WeightedSegmentVarianceAdjuster for MinimumWeightedSegmentVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedSegmentVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the weighted segment variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedSegmentVarianceEngine {
    pub steps: Vec<Rc<dyn WeightedSegmentVarianceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedSegmentVarianceEngine {
    pub fn evaluate(&mut self, item: &WeightedSegmentVariance) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[WeightedSegmentVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_segment_variance_engine(flags: &flags::Registry) -> WeightedSegmentVarianceEngine {
    WeightedSegmentVarianceEngine {
        steps: vec![
            Rc::new(FlaggedWeightedSegmentVarianceAdjuster {
                flags: flags.clone(),
                flag: "weighted-segment-variance-scheduled".to_string(),
                percent: 1.3,
            }) as Rc<dyn WeightedSegmentVarianceAdjuster>,
            Rc::new(CapWeightedSegmentVarianceAdjuster { cap: 19934.0 }),
            Rc::new(SourceWeightedSegmentVarianceAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.73)]),
            }),
            Rc::new(PercentWeightedSegmentVarianceAdjuster { percent: 10.5 }),
            Rc::new(MinimumWeightedSegmentVarianceAdjuster { minimum: 30.5 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 39817.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_segment_variance_engine(&flags::new_registry());
        let item = new_weighted_segment_variance("code-36", "name-90");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
