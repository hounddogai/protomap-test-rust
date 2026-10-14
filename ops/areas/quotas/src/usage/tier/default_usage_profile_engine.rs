use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default usage profile calculation.
pub trait DefaultUsageProfileFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultUsageProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultUsageProfileAdjuster {
    pub step: f64,
}

impl DefaultUsageProfileFactor for RoundingDefaultUsageProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultUsageProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultUsageProfileAdjuster {
    pub minimum: f64,
}

impl DefaultUsageProfileFactor for MinimumDefaultUsageProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultUsageProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultUsageProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultUsageProfileFactor for FlaggedDefaultUsageProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultUsageProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterDefaultUsageProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultUsageProfileFactor for ClusterDefaultUsageProfileAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &DefaultUsageProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageProfileCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl DefaultUsageProfileFactor for DefaultUsageProfileCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultUsageProfile, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

/// Runs the default usage profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultUsageProfileEngine {
    pub steps: Vec<Rc<dyn DefaultUsageProfileFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultUsageProfileEngine {
    pub fn compute(&mut self, item: &DefaultUsageProfile) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[DefaultUsageProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_usage_profile_engine(flags: &flags::Registry) -> DefaultUsageProfileEngine {
    DefaultUsageProfileEngine {
        steps: vec![
            Rc::new(RoundingDefaultUsageProfileAdjuster { step: 5.0 }) as Rc<dyn DefaultUsageProfileFactor>,
            Rc::new(MinimumDefaultUsageProfileAdjuster { minimum: 13.7 }),
            Rc::new(FlaggedDefaultUsageProfileAdjuster {
                flags: flags.clone(),
                flag: "default-usage-profile-rolling".to_string(),
                percent: 6.0,
            }),
            Rc::new(ClusterDefaultUsageProfileAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.32), ("north".to_string(), 0.73)]),
            }),
            Rc::new(DefaultUsageProfileCreatedAtOffsetAdjuster { weight: 0.64 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 41432.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_usage_profile_engine(&flags::new_registry());
        let item = new_default_usage_profile("code-332", "name-762");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
