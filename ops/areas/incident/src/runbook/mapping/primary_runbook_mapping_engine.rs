use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary runbook mapping calculation.
pub trait PrimaryRunbookMappingFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryRunbookMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryRunbookMappingAdjuster {
    pub minimum: f64,
}

impl PrimaryRunbookMappingFactor for MinimumPrimaryRunbookMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryRunbookMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryRunbookMappingAdjuster {
    pub step: f64,
}

impl PrimaryRunbookMappingFactor for RoundingPrimaryRunbookMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryRunbookMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRunbookMappingPortOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryRunbookMappingFactor for PrimaryRunbookMappingPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryRunbookMapping, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalePrimaryRunbookMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryRunbookMappingFactor for LocalePrimaryRunbookMappingAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &PrimaryRunbookMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryRunbookMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryRunbookMappingFactor for FlaggedPrimaryRunbookMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryRunbookMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary runbook mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryRunbookMappingEngine {
    pub steps: Vec<Rc<dyn PrimaryRunbookMappingFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryRunbookMappingEngine {
    pub fn compute(&mut self, item: &PrimaryRunbookMapping) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryRunbookMappingFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryRunbookMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_runbook_mapping_engine(flags: &flags::Registry) -> PrimaryRunbookMappingEngine {
    PrimaryRunbookMappingEngine {
        steps: vec![
            Rc::new(MinimumPrimaryRunbookMappingAdjuster { minimum: 20.1 }) as Rc<dyn PrimaryRunbookMappingFactor>,
            Rc::new(RoundingPrimaryRunbookMappingAdjuster { step: 0.05 }),
            Rc::new(PrimaryRunbookMappingPortOffsetAdjuster { weight: 0.82 }),
            Rc::new(LocalePrimaryRunbookMappingAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.75)]),
            }),
            Rc::new(FlaggedPrimaryRunbookMappingAdjuster {
                flags: flags.clone(),
                flag: "primary-runbook-mapping-legacy".to_string(),
                percent: 0.7,
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 45566.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_runbook_mapping_engine(&flags::new_registry());
        let item = new_primary_runbook_mapping("code-433", "label-805");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
