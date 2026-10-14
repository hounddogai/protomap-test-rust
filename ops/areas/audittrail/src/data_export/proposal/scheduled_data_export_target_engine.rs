use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled data export target calculation.
pub trait ScheduledDataExportTargetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledDataExportTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledDataExportTargetAdjuster {
    pub minimum: f64,
}

impl ScheduledDataExportTargetModifier for MinimumScheduledDataExportTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledDataExportTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledDataExportTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledDataExportTargetModifier for TieredScheduledDataExportTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledDataExportTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledDataExportTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledDataExportTargetModifier for FlaggedScheduledDataExportTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledDataExportTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportTargetScoreOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledDataExportTargetModifier for ScheduledDataExportTargetScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledDataExportTarget, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledDataExportTargetAdjuster {
    pub step: f64,
}

impl ScheduledDataExportTargetModifier for RoundingScheduledDataExportTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledDataExportTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterScheduledDataExportTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledDataExportTargetModifier for ClusterScheduledDataExportTargetAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &ScheduledDataExportTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled data export target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledDataExportTargetEngine {
    pub steps: Vec<Rc<dyn ScheduledDataExportTargetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledDataExportTargetEngine {
    pub fn compute(&mut self, item: &ScheduledDataExportTarget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledDataExportTargetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledDataExportTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_data_export_target_engine(flags: &flags::Registry) -> ScheduledDataExportTargetEngine {
    ScheduledDataExportTargetEngine {
        steps: vec![
            Rc::new(MinimumScheduledDataExportTargetAdjuster { minimum: 26.3 })
                as Rc<dyn ScheduledDataExportTargetModifier>,
            Rc::new(TieredScheduledDataExportTargetAdjuster {
                bounds: vec![5749.0, 8990.0],
                rates: vec![1.04, 0.97],
            }),
            Rc::new(FlaggedScheduledDataExportTargetAdjuster {
                flags: flags.clone(),
                flag: "scheduled-data-export-target-batched".to_string(),
                percent: -5.4,
            }),
            Rc::new(ScheduledDataExportTargetScoreOffsetAdjuster { weight: 1.02 }),
            Rc::new(RoundingScheduledDataExportTargetAdjuster { step: 0.05 }),
            Rc::new(ClusterScheduledDataExportTargetAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.38),
                    ("online".to_string(), 0.86),
                    ("east".to_string(), 1.24),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 3415.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_data_export_target_engine(&flags::new_registry());
        let item = new_scheduled_data_export_target("code-181", "title-720");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
