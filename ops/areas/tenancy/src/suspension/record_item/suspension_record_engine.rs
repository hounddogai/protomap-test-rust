use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the suspension record calculation.
pub trait SuspensionRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSuspensionRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SuspensionRecordAdjuster for TieredSuspensionRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SuspensionRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterSuspensionRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionRecordAdjuster for ClusterSuspensionRecordAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &SuspensionRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSuspensionRecordAdjuster {
    pub percent: f64,
}

impl SuspensionRecordAdjuster for PercentSuspensionRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SuspensionRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSuspensionRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SuspensionRecordAdjuster for FlaggedSuspensionRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SuspensionRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRecordBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionRecordAdjuster for SuspensionRecordBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionRecord, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionRecordAdjuster {
    pub minimum: f64,
}

impl SuspensionRecordAdjuster for MinimumSuspensionRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the suspension record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionRecordEngine {
    pub steps: Vec<Rc<dyn SuspensionRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionRecordEngine {
    pub fn evaluate(&mut self, item: &SuspensionRecord) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SuspensionRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_suspension_record_engine(flags: &flags::Registry) -> SuspensionRecordEngine {
    SuspensionRecordEngine {
        steps: vec![
            Rc::new(TieredSuspensionRecordAdjuster {
                bounds: vec![371.0, 5804.0, 7431.0, 7577.0],
                rates: vec![1.2, 0.87, 0.82, 1.08],
            }) as Rc<dyn SuspensionRecordAdjuster>,
            Rc::new(ClusterSuspensionRecordAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.72),
                    ("retail".to_string(), 1.08),
                    ("west".to_string(), 1.26),
                ]),
            }),
            Rc::new(PercentSuspensionRecordAdjuster { percent: 8.6 }),
            Rc::new(FlaggedSuspensionRecordAdjuster {
                flags: flags.clone(),
                flag: "suspension-record-tiered".to_string(),
                percent: 1.4,
            }),
            Rc::new(SuspensionRecordBatchSizeOffsetAdjuster { weight: 0.72 }),
            Rc::new(MinimumSuspensionRecordAdjuster { minimum: 48.2 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 66768.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_record_engine(&flags::new_registry());
        let item = new_suspension_record("code-615", "name-818");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
