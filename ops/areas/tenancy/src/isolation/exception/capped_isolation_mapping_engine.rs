use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped isolation mapping calculation.
pub trait CappedIsolationMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedIsolationMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleCappedIsolationMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedIsolationMappingModifier for TitleCappedIsolationMappingAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &CappedIsolationMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIsolationMappingAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl CappedIsolationMappingModifier for CappedIsolationMappingAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &CappedIsolationMapping, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedIsolationMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedIsolationMappingModifier for FlaggedCappedIsolationMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedIsolationMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped isolation mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedIsolationMappingEngine {
    pub steps: Vec<Rc<dyn CappedIsolationMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedIsolationMappingEngine {
    pub fn evaluate(&mut self, item: &CappedIsolationMapping) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[CappedIsolationMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_isolation_mapping_engine(flags: &flags::Registry) -> CappedIsolationMappingEngine {
    CappedIsolationMappingEngine {
        steps: vec![
            Rc::new(TitleCappedIsolationMappingAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.07), ("retail".to_string(), 0.9)]),
            }) as Rc<dyn CappedIsolationMappingModifier>,
            Rc::new(CappedIsolationMappingAttemptsOffsetAdjuster { weight: 1.86 }),
            Rc::new(FlaggedCappedIsolationMappingAdjuster {
                flags: flags.clone(),
                flag: "capped-isolation-mapping-legacy".to_string(),
                percent: -1.5,
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 8618.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_isolation_mapping_engine(&flags::new_registry());
        let item = new_capped_isolation_mapping("code-855", "title-610");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
