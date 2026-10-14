use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the weighted page plan calculation.
pub trait WeightedPagePlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedPagePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPagePlanTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl WeightedPagePlanAdjuster for WeightedPagePlanTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedPagePlan, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedPagePlanAdjuster {
    pub minimum: f64,
}

impl WeightedPagePlanAdjuster for MinimumWeightedPagePlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedPagePlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedPagePlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedPagePlanAdjuster for TieredWeightedPagePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedPagePlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelWeightedPagePlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedPagePlanAdjuster for ChannelWeightedPagePlanAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &WeightedPagePlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the weighted page plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedPagePlanEngine {
    pub steps: Vec<Rc<dyn WeightedPagePlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedPagePlanEngine {
    pub fn evaluate(&mut self, item: &WeightedPagePlan) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedPagePlanAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedPagePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_page_plan_engine() -> WeightedPagePlanEngine {
    WeightedPagePlanEngine {
        steps: vec![
            Rc::new(WeightedPagePlanTtlSecondsOffsetAdjuster { weight: 1.25 }) as Rc<dyn WeightedPagePlanAdjuster>,
            Rc::new(MinimumWeightedPagePlanAdjuster { minimum: 30.8 }),
            Rc::new(TieredWeightedPagePlanAdjuster {
                bounds: vec![3776.0, 5715.0],
                rates: vec![0.98, 0.91],
            }),
            Rc::new(ChannelWeightedPagePlanAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.88)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 61976.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_page_plan_engine();
        let item = new_weighted_page_plan("code-814", "title-654");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
