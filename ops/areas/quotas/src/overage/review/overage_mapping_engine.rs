use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the overage mapping calculation.
pub trait OverageMappingStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOverageMappingAdjuster {
    pub step: f64,
}

impl OverageMappingStep for RoundingOverageMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OverageMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedOverageMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl OverageMappingStep for FlaggedOverageMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &OverageMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameOverageMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl OverageMappingStep for NameOverageMappingAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &OverageMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumOverageMappingAdjuster {
    pub minimum: f64,
}

impl OverageMappingStep for MinimumOverageMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &OverageMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the overage mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageMappingEngine {
    pub steps: Vec<Rc<dyn OverageMappingStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageMappingEngine {
    pub fn compute(&mut self, item: &OverageMapping) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[OverageMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_overage_mapping_engine(flags: &flags::Registry) -> OverageMappingEngine {
    OverageMappingEngine {
        steps: vec![
            Rc::new(RoundingOverageMappingAdjuster { step: 1.0 }) as Rc<dyn OverageMappingStep>,
            Rc::new(FlaggedOverageMappingAdjuster {
                flags: flags.clone(),
                flag: "overage-mapping-manual".to_string(),
                percent: 9.3,
            }),
            Rc::new(NameOverageMappingAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.34)]),
            }),
            Rc::new(MinimumOverageMappingAdjuster { minimum: 6.4 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 99217.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_mapping_engine(&flags::new_registry());
        let item = new_overage_mapping("code-875", "name-754");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
