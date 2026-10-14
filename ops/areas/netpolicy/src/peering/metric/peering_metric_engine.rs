use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the peering metric calculation.
pub trait PeeringMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPeeringMetricAdjuster {
    pub minimum: f64,
}

impl PeeringMetricAdjuster for MinimumPeeringMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PeeringMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringMetricDiscountOffsetAdjuster {
    pub weight: f64,
}

impl PeeringMetricAdjuster for PeeringMetricDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringMetric, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryPeeringMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PeeringMetricAdjuster for CategoryPeeringMetricAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &PeeringMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPeeringMetricAdjuster {
    pub percent: f64,
}

impl PeeringMetricAdjuster for PercentPeeringMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PeeringMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the peering metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringMetricEngine {
    pub steps: Vec<Rc<dyn PeeringMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringMetricEngine {
    pub fn run(&mut self, item: &PeeringMetric) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PeeringMetricAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PeeringMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_peering_metric_engine() -> PeeringMetricEngine {
    PeeringMetricEngine {
        steps: vec![
            Rc::new(MinimumPeeringMetricAdjuster { minimum: 31.8 }) as Rc<dyn PeeringMetricAdjuster>,
            Rc::new(PeeringMetricDiscountOffsetAdjuster { weight: 1.82 }),
            Rc::new(CategoryPeeringMetricAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.98), ("online".to_string(), 0.71)]),
            }),
            Rc::new(PercentPeeringMetricAdjuster { percent: 5.9 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 33064.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_metric_engine();
        let item = new_peering_metric("code-9", "name-578");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
