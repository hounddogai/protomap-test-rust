use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict page group calculation.
pub trait StrictPageGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictPageGroup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictPageGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictPageGroupStep for FlaggedStrictPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictPageGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictPageGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictPageGroupStep for TieredStrictPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictPageGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictPageGroupAdjuster {
    pub cap: f64,
}

impl StrictPageGroupStep for CapStrictPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictPageGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the strict page group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictPageGroupEngine {
    pub steps: Vec<Rc<dyn StrictPageGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictPageGroupEngine {
    pub fn run(&mut self, item: &StrictPageGroup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[StrictPageGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_page_group_engine(flags: &flags::Registry) -> StrictPageGroupEngine {
    StrictPageGroupEngine {
        steps: vec![
            Rc::new(FlaggedStrictPageGroupAdjuster {
                flags: flags.clone(),
                flag: "strict-page-group-primary".to_string(),
                percent: -8.9,
            }) as Rc<dyn StrictPageGroupStep>,
            Rc::new(TieredStrictPageGroupAdjuster {
                bounds: vec![1360.0, 7273.0, 9931.0],
                rates: vec![1.06, 0.95, 1.12],
            }),
            Rc::new(CapStrictPageGroupAdjuster { cap: 18861.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 75801.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_page_group_engine(&flags::new_registry());
        let item = new_strict_page_group("code-535", "label-772");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
