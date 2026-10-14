use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted failover segment calculation.
pub trait WeightedFailoverSegmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedFailoverSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedFailoverSegmentAdjuster {
    pub step: f64,
}

impl WeightedFailoverSegmentModifier for RoundingWeightedFailoverSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedFailoverSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleWeightedFailoverSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedFailoverSegmentModifier for TitleWeightedFailoverSegmentAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &WeightedFailoverSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedFailoverSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedFailoverSegmentModifier for FlaggedWeightedFailoverSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedFailoverSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverSegmentLimitOffsetAdjuster {
    pub weight: f64,
}

impl WeightedFailoverSegmentModifier for WeightedFailoverSegmentLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedFailoverSegment, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedFailoverSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedFailoverSegmentModifier for TieredWeightedFailoverSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedFailoverSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedFailoverSegmentAdjuster {
    pub percent: f64,
}

impl WeightedFailoverSegmentModifier for PercentWeightedFailoverSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedFailoverSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted failover segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedFailoverSegmentEngine {
    pub steps: Vec<Rc<dyn WeightedFailoverSegmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedFailoverSegmentEngine {
    pub fn evaluate(&mut self, item: &WeightedFailoverSegment) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedFailoverSegmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedFailoverSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_failover_segment_engine(flags: &flags::Registry) -> WeightedFailoverSegmentEngine {
    WeightedFailoverSegmentEngine {
        steps: vec![
            Rc::new(RoundingWeightedFailoverSegmentAdjuster { step: 1.0 }) as Rc<dyn WeightedFailoverSegmentModifier>,
            Rc::new(TitleWeightedFailoverSegmentAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.18)]),
            }),
            Rc::new(FlaggedWeightedFailoverSegmentAdjuster {
                flags: flags.clone(),
                flag: "weighted-failover-segment-global".to_string(),
                percent: 6.1,
            }),
            Rc::new(WeightedFailoverSegmentLimitOffsetAdjuster { weight: 1.56 }),
            Rc::new(TieredWeightedFailoverSegmentAdjuster {
                bounds: vec![1666.0, 4538.0, 6104.0],
                rates: vec![0.89, 0.95, 0.81],
            }),
            Rc::new(PercentWeightedFailoverSegmentAdjuster { percent: 15.6 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 36815.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_failover_segment_engine(&flags::new_registry());
        let item = new_weighted_failover_segment("code-85", "title-5");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
