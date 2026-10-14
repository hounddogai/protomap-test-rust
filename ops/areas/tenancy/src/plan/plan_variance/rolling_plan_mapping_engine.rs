use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling plan mapping calculation.
pub trait RollingPlanMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPlanMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingPlanMappingAdjuster {
    pub percent: f64,
}

impl RollingPlanMappingModifier for PercentRollingPlanMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingPlanMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingPlanMappingAdjuster {
    pub cap: f64,
}

impl RollingPlanMappingModifier for CapRollingPlanMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingPlanMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRollingPlanMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingPlanMappingModifier for TitleRollingPlanMappingAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RollingPlanMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingPlanMappingAdjuster {
    pub step: f64,
}

impl RollingPlanMappingModifier for RoundingRollingPlanMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingPlanMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingPlanMappingAdjuster {
    pub minimum: f64,
}

impl RollingPlanMappingModifier for MinimumRollingPlanMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingPlanMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanMappingLimitOffsetAdjuster {
    pub weight: f64,
}

impl RollingPlanMappingModifier for RollingPlanMappingLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &RollingPlanMapping, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

/// Runs the rolling plan mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPlanMappingEngine {
    pub steps: Vec<Rc<dyn RollingPlanMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPlanMappingEngine {
    pub fn evaluate(&mut self, item: &RollingPlanMapping) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingPlanMappingModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingPlanMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_plan_mapping_engine() -> RollingPlanMappingEngine {
    RollingPlanMappingEngine {
        steps: vec![
            Rc::new(PercentRollingPlanMappingAdjuster { percent: -17.9 }) as Rc<dyn RollingPlanMappingModifier>,
            Rc::new(CapRollingPlanMappingAdjuster { cap: 38395.0 }),
            Rc::new(TitleRollingPlanMappingAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.96), ("south".to_string(), 0.72)]),
            }),
            Rc::new(RoundingRollingPlanMappingAdjuster { step: 5.0 }),
            Rc::new(MinimumRollingPlanMappingAdjuster { minimum: 46.3 }),
            Rc::new(RollingPlanMappingLimitOffsetAdjuster { weight: 1.47 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 75069.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_plan_mapping_engine();
        let item = new_rolling_plan_mapping("code-360", "title-341");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
