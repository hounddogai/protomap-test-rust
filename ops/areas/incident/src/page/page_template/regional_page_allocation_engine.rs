use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional page allocation calculation.
pub trait RegionalPageAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalPageAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRegionalPageAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalPageAllocationModifier for NameRegionalPageAllocationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RegionalPageAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalPageAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalPageAllocationModifier for FlaggedRegionalPageAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalPageAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalPageAllocationAdjuster {
    pub step: f64,
}

impl RegionalPageAllocationModifier for RoundingRegionalPageAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalPageAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalPageAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalPageAllocationModifier for TieredRegionalPageAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalPageAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the regional page allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalPageAllocationEngine {
    pub steps: Vec<Rc<dyn RegionalPageAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalPageAllocationEngine {
    pub fn evaluate(&mut self, item: &RegionalPageAllocation) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RegionalPageAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_page_allocation_engine(flags: &flags::Registry) -> RegionalPageAllocationEngine {
    RegionalPageAllocationEngine {
        steps: vec![
            Rc::new(NameRegionalPageAllocationAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.78)]),
            }) as Rc<dyn RegionalPageAllocationModifier>,
            Rc::new(FlaggedRegionalPageAllocationAdjuster {
                flags: flags.clone(),
                flag: "regional-page-allocation-rolling".to_string(),
                percent: -7.1,
            }),
            Rc::new(RoundingRegionalPageAllocationAdjuster { step: 0.25 }),
            Rc::new(TieredRegionalPageAllocationAdjuster {
                bounds: vec![439.0, 1309.0],
                rates: vec![1.07, 1.01],
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 68108.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_page_allocation_engine(&flags::new_registry());
        let item = new_regional_page_allocation("code-409", "name-470");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
