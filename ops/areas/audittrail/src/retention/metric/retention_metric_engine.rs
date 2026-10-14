use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the retention metric calculation.
pub trait RetentionMetricModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionMetricAdjuster {
    pub minimum: f64,
}

impl RetentionMetricModifier for MinimumRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionMetricModifier for FlaggedRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRetentionMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionMetricModifier for SegmentRetentionMetricAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &RetentionMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RetentionMetricModifier for TieredRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RetentionMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMetricGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl RetentionMetricModifier for RetentionMetricGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionMetric, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionMetricAdjuster {
    pub cap: f64,
}

impl RetentionMetricModifier for CapRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the retention metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionMetricEngine {
    pub steps: Vec<Rc<dyn RetentionMetricModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionMetricEngine {
    pub fn price(&mut self, item: &RetentionMetric) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionMetricModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_retention_metric_engine(flags: &flags::Registry) -> RetentionMetricEngine {
    RetentionMetricEngine {
        steps: vec![
            Rc::new(MinimumRetentionMetricAdjuster { minimum: 7.8 }) as Rc<dyn RetentionMetricModifier>,
            Rc::new(FlaggedRetentionMetricAdjuster {
                flags: flags.clone(),
                flag: "retention-metric-rolling".to_string(),
                percent: 9.8,
            }),
            Rc::new(SegmentRetentionMetricAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.17), ("north".to_string(), 1.25)]),
            }),
            Rc::new(TieredRetentionMetricAdjuster {
                bounds: vec![2181.0, 4988.0, 8597.0, 9008.0],
                rates: vec![0.82, 1.04, 1.08, 0.88],
            }),
            Rc::new(RetentionMetricGraceDaysOffsetAdjuster { weight: 1.96 }),
            Rc::new(CapRetentionMetricAdjuster { cap: 39339.0 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 44518.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_metric_engine(&flags::new_registry());
        let item = new_retention_metric("code-876", "label-181");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
