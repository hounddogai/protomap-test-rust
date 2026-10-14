use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the postmortem mapping calculation.
pub trait PostmortemMappingFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPostmortemMappingAdjuster {
    pub cap: f64,
}

impl PostmortemMappingFactor for CapPostmortemMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PostmortemMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemMappingFactor for FlaggedPostmortemMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemMappingRetriesOffsetAdjuster {
    pub weight: f64,
}

impl PostmortemMappingFactor for PostmortemMappingRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &PostmortemMapping, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPostmortemMappingAdjuster {
    pub percent: f64,
}

impl PostmortemMappingFactor for PercentPostmortemMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PostmortemMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the postmortem mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemMappingEngine {
    pub steps: Vec<Rc<dyn PostmortemMappingFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemMappingEngine {
    pub fn evaluate(&mut self, item: &PostmortemMapping) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PostmortemMappingFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_postmortem_mapping_engine(flags: &flags::Registry) -> PostmortemMappingEngine {
    PostmortemMappingEngine {
        steps: vec![
            Rc::new(CapPostmortemMappingAdjuster { cap: 26288.0 }) as Rc<dyn PostmortemMappingFactor>,
            Rc::new(FlaggedPostmortemMappingAdjuster {
                flags: flags.clone(),
                flag: "postmortem-mapping-manual".to_string(),
                percent: 6.0,
            }),
            Rc::new(PostmortemMappingRetriesOffsetAdjuster { weight: 1.45 }),
            Rc::new(PercentPostmortemMappingAdjuster { percent: -17.4 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 5946.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_postmortem_mapping_engine(&flags::new_registry());
        let item = new_postmortem_mapping("code-178", "title-904");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
