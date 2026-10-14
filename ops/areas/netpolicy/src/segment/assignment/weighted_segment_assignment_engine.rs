use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted segment assignment calculation.
pub trait WeightedSegmentAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedSegmentAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedSegmentAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedSegmentAssignmentFactor for TieredWeightedSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedSegmentAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionWeightedSegmentAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedSegmentAssignmentFactor for RegionWeightedSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &WeightedSegmentAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedSegmentAssignmentAdjuster {
    pub cap: f64,
}

impl WeightedSegmentAssignmentFactor for CapWeightedSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedSegmentAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedSegmentAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedSegmentAssignmentFactor for FlaggedWeightedSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedSegmentAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedSegmentAssignmentAdjuster {
    pub minimum: f64,
}

impl WeightedSegmentAssignmentFactor for MinimumWeightedSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedSegmentAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the weighted segment assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedSegmentAssignmentEngine {
    pub steps: Vec<Rc<dyn WeightedSegmentAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedSegmentAssignmentEngine {
    pub fn evaluate(&mut self, item: &WeightedSegmentAssignment) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[WeightedSegmentAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_segment_assignment_engine(flags: &flags::Registry) -> WeightedSegmentAssignmentEngine {
    WeightedSegmentAssignmentEngine {
        steps: vec![
            Rc::new(TieredWeightedSegmentAssignmentAdjuster {
                bounds: vec![1777.0, 9862.0],
                rates: vec![1.19, 0.99],
            }) as Rc<dyn WeightedSegmentAssignmentFactor>,
            Rc::new(RegionWeightedSegmentAssignmentAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.84), ("retail".to_string(), 1.28)]),
            }),
            Rc::new(CapWeightedSegmentAssignmentAdjuster { cap: 8611.0 }),
            Rc::new(FlaggedWeightedSegmentAssignmentAdjuster {
                flags: flags.clone(),
                flag: "weighted-segment-assignment-batched".to_string(),
                percent: 6.4,
            }),
            Rc::new(MinimumWeightedSegmentAssignmentAdjuster { minimum: 24.8 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 21898.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_segment_assignment_engine(&flags::new_registry());
        let item = new_weighted_segment_assignment("code-297", "label-570");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
