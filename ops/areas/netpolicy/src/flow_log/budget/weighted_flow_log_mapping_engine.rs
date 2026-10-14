use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted flow log mapping calculation.
pub trait WeightedFlowLogMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedFlowLogMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFlowLogMappingPriorityOffsetAdjuster {
    pub weight: f64,
}

impl WeightedFlowLogMappingModifier for WeightedFlowLogMappingPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedFlowLogMapping, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedFlowLogMappingAdjuster {
    pub percent: f64,
}

impl WeightedFlowLogMappingModifier for PercentWeightedFlowLogMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedFlowLogMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedFlowLogMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedFlowLogMappingModifier for FlaggedWeightedFlowLogMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedFlowLogMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedFlowLogMappingAdjuster {
    pub minimum: f64,
}

impl WeightedFlowLogMappingModifier for MinimumWeightedFlowLogMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedFlowLogMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the weighted flow log mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedFlowLogMappingEngine {
    pub steps: Vec<Rc<dyn WeightedFlowLogMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedFlowLogMappingEngine {
    pub fn compute(&mut self, item: &WeightedFlowLogMapping) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedFlowLogMappingModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedFlowLogMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_flow_log_mapping_engine(flags: &flags::Registry) -> WeightedFlowLogMappingEngine {
    WeightedFlowLogMappingEngine {
        steps: vec![
            Rc::new(WeightedFlowLogMappingPriorityOffsetAdjuster { weight: 1.16 })
                as Rc<dyn WeightedFlowLogMappingModifier>,
            Rc::new(PercentWeightedFlowLogMappingAdjuster { percent: -10.2 }),
            Rc::new(FlaggedWeightedFlowLogMappingAdjuster {
                flags: flags.clone(),
                flag: "weighted-flow-log-mapping-rolling".to_string(),
                percent: 9.4,
            }),
            Rc::new(MinimumWeightedFlowLogMappingAdjuster { minimum: 26.9 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 3108.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_flow_log_mapping_engine(&flags::new_registry());
        let item = new_weighted_flow_log_mapping("code-609", "name-730");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
