use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary failover group calculation.
pub trait PrimaryFailoverGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryFailoverGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryFailoverGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryFailoverGroupStep for TieredPrimaryFailoverGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryFailoverGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPrimaryFailoverGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryFailoverGroupStep for LabelPrimaryFailoverGroupAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PrimaryFailoverGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryFailoverGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryFailoverGroupStep for FlaggedPrimaryFailoverGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryFailoverGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary failover group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryFailoverGroupEngine {
    pub steps: Vec<Rc<dyn PrimaryFailoverGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryFailoverGroupEngine {
    pub fn price(&mut self, item: &PrimaryFailoverGroup) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryFailoverGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_failover_group_engine(flags: &flags::Registry) -> PrimaryFailoverGroupEngine {
    PrimaryFailoverGroupEngine {
        steps: vec![
            Rc::new(TieredPrimaryFailoverGroupAdjuster {
                bounds: vec![237.0, 1868.0, 1903.0],
                rates: vec![0.94, 0.9, 1.16],
            }) as Rc<dyn PrimaryFailoverGroupStep>,
            Rc::new(LabelPrimaryFailoverGroupAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.77)]),
            }),
            Rc::new(FlaggedPrimaryFailoverGroupAdjuster {
                flags: flags.clone(),
                flag: "primary-failover-group-adaptive".to_string(),
                percent: 4.6,
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 31298.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_failover_group_engine(&flags::new_registry());
        let item = new_primary_failover_group("code-5", "label-347");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
