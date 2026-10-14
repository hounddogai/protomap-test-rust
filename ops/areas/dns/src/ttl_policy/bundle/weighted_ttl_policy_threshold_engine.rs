use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the weighted ttl policy threshold calculation.
pub trait WeightedTtlPolicyThresholdFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedTtlPolicyThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyThresholdRateOffsetAdjuster {
    pub weight: f64,
}

impl WeightedTtlPolicyThresholdFactor for WeightedTtlPolicyThresholdRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedTtlPolicyThreshold, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedTtlPolicyThresholdAdjuster {
    pub minimum: f64,
}

impl WeightedTtlPolicyThresholdFactor for MinimumWeightedTtlPolicyThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedTtlPolicyThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceWeightedTtlPolicyThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedTtlPolicyThresholdFactor for SourceWeightedTtlPolicyThresholdAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &WeightedTtlPolicyThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedTtlPolicyThresholdAdjuster {
    pub cap: f64,
}

impl WeightedTtlPolicyThresholdFactor for CapWeightedTtlPolicyThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedTtlPolicyThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedTtlPolicyThresholdAdjuster {
    pub percent: f64,
}

impl WeightedTtlPolicyThresholdFactor for PercentWeightedTtlPolicyThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedTtlPolicyThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted ttl policy threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedTtlPolicyThresholdEngine {
    pub steps: Vec<Rc<dyn WeightedTtlPolicyThresholdFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedTtlPolicyThresholdEngine {
    pub fn compute(&mut self, item: &WeightedTtlPolicyThreshold) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedTtlPolicyThresholdFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedTtlPolicyThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_ttl_policy_threshold_engine() -> WeightedTtlPolicyThresholdEngine {
    WeightedTtlPolicyThresholdEngine {
        steps: vec![
            Rc::new(WeightedTtlPolicyThresholdRateOffsetAdjuster { weight: 1.46 })
                as Rc<dyn WeightedTtlPolicyThresholdFactor>,
            Rc::new(MinimumWeightedTtlPolicyThresholdAdjuster { minimum: 35.2 }),
            Rc::new(SourceWeightedTtlPolicyThresholdAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.02),
                    ("north".to_string(), 1.12),
                    ("east".to_string(), 0.88),
                ]),
            }),
            Rc::new(CapWeightedTtlPolicyThresholdAdjuster { cap: 27264.0 }),
            Rc::new(PercentWeightedTtlPolicyThresholdAdjuster { percent: 6.1 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 26211.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_ttl_policy_threshold_engine();
        let item = new_weighted_ttl_policy_threshold("code-496", "label-717");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
