use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the capped query mapping calculation.
pub trait CappedQueryMappingStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedQueryMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedQueryMappingAdjuster {
    pub minimum: f64,
}

impl CappedQueryMappingStep for MinimumCappedQueryMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedQueryMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedQueryMappingAdjuster {
    pub cap: f64,
}

impl CappedQueryMappingStep for CapCappedQueryMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedQueryMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedQueryMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedQueryMappingStep for FlaggedCappedQueryMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedQueryMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped query mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedQueryMappingEngine {
    pub steps: Vec<Rc<dyn CappedQueryMappingStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedQueryMappingEngine {
    pub fn evaluate(&mut self, item: &CappedQueryMapping) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedQueryMappingStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedQueryMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_query_mapping_engine(flags: &flags::Registry) -> CappedQueryMappingEngine {
    CappedQueryMappingEngine {
        steps: vec![
            Rc::new(MinimumCappedQueryMappingAdjuster { minimum: 13.2 }) as Rc<dyn CappedQueryMappingStep>,
            Rc::new(CapCappedQueryMappingAdjuster { cap: 23370.0 }),
            Rc::new(FlaggedCappedQueryMappingAdjuster {
                flags: flags.clone(),
                flag: "capped-query-mapping-lenient".to_string(),
                percent: 6.0,
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 97510.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_query_mapping_engine(&flags::new_registry());
        let item = new_capped_query_mapping("code-963", "name-324");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
