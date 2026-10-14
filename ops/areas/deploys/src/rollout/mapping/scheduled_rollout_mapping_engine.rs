use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the scheduled rollout mapping calculation.
pub trait ScheduledRolloutMappingFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledRolloutMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryScheduledRolloutMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledRolloutMappingFactor for CategoryScheduledRolloutMappingAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &ScheduledRolloutMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledRolloutMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledRolloutMappingFactor for FlaggedScheduledRolloutMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledRolloutMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledRolloutMappingAdjuster {
    pub cap: f64,
}

impl ScheduledRolloutMappingFactor for CapScheduledRolloutMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledRolloutMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled rollout mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledRolloutMappingEngine {
    pub steps: Vec<Rc<dyn ScheduledRolloutMappingFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledRolloutMappingEngine {
    pub fn price(&mut self, item: &ScheduledRolloutMapping) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledRolloutMappingFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledRolloutMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_rollout_mapping_engine(flags: &flags::Registry) -> ScheduledRolloutMappingEngine {
    ScheduledRolloutMappingEngine {
        steps: vec![
            Rc::new(CategoryScheduledRolloutMappingAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.98),
                    ("central".to_string(), 0.97),
                    ("west".to_string(), 1.22),
                ]),
            }) as Rc<dyn ScheduledRolloutMappingFactor>,
            Rc::new(FlaggedScheduledRolloutMappingAdjuster {
                flags: flags.clone(),
                flag: "scheduled-rollout-mapping-lenient".to_string(),
                percent: 5.2,
            }),
            Rc::new(CapScheduledRolloutMappingAdjuster { cap: 5246.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 74024.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_rollout_mapping_engine(&flags::new_registry());
        let item = new_scheduled_rollout_mapping("code-496", "title-963");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
