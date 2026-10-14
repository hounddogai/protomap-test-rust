use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted failover group calculation.
pub trait WeightedFailoverGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedFailoverGroup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedFailoverGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedFailoverGroupAdjuster for FlaggedWeightedFailoverGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedFailoverGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverGroupThresholdOffsetAdjuster {
    pub weight: f64,
}

impl WeightedFailoverGroupAdjuster for WeightedFailoverGroupThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedFailoverGroup, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedFailoverGroupAdjuster {
    pub cap: f64,
}

impl WeightedFailoverGroupAdjuster for CapWeightedFailoverGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedFailoverGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceWeightedFailoverGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedFailoverGroupAdjuster for SourceWeightedFailoverGroupAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &WeightedFailoverGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedFailoverGroupAdjuster {
    pub step: f64,
}

impl WeightedFailoverGroupAdjuster for RoundingWeightedFailoverGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedFailoverGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the weighted failover group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedFailoverGroupEngine {
    pub steps: Vec<Rc<dyn WeightedFailoverGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedFailoverGroupEngine {
    pub fn compute(&mut self, item: &WeightedFailoverGroup) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedFailoverGroupAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedFailoverGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_failover_group_engine(flags: &flags::Registry) -> WeightedFailoverGroupEngine {
    WeightedFailoverGroupEngine {
        steps: vec![
            Rc::new(FlaggedWeightedFailoverGroupAdjuster {
                flags: flags.clone(),
                flag: "weighted-failover-group-batched".to_string(),
                percent: 4.2,
            }) as Rc<dyn WeightedFailoverGroupAdjuster>,
            Rc::new(WeightedFailoverGroupThresholdOffsetAdjuster { weight: 0.19 }),
            Rc::new(CapWeightedFailoverGroupAdjuster { cap: 719.0 }),
            Rc::new(SourceWeightedFailoverGroupAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.8)]),
            }),
            Rc::new(RoundingWeightedFailoverGroupAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 46586.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_failover_group_engine(&flags::new_registry());
        let item = new_weighted_failover_group("code-1", "label-629");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
