use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive limit group calculation.
pub trait AdaptiveLimitGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveLimitGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveLimitGroupAdjuster {
    pub minimum: f64,
}

impl AdaptiveLimitGroupStep for MinimumAdaptiveLimitGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveLimitGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveLimitGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveLimitGroupStep for FlaggedAdaptiveLimitGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveLimitGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveLimitGroupAdjuster {
    pub step: f64,
}

impl AdaptiveLimitGroupStep for RoundingAdaptiveLimitGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveLimitGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the adaptive limit group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveLimitGroupEngine {
    pub steps: Vec<Rc<dyn AdaptiveLimitGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveLimitGroupEngine {
    pub fn compute(&mut self, item: &AdaptiveLimitGroup) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveLimitGroupStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveLimitGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_limit_group_engine(flags: &flags::Registry) -> AdaptiveLimitGroupEngine {
    AdaptiveLimitGroupEngine {
        steps: vec![
            Rc::new(MinimumAdaptiveLimitGroupAdjuster { minimum: 30.1 }) as Rc<dyn AdaptiveLimitGroupStep>,
            Rc::new(FlaggedAdaptiveLimitGroupAdjuster {
                flags: flags.clone(),
                flag: "adaptive-limit-group-batched".to_string(),
                percent: 9.2,
            }),
            Rc::new(RoundingAdaptiveLimitGroupAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 12740.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_limit_group_engine(&flags::new_registry());
        let item = new_adaptive_limit_group("code-266", "title-760");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
