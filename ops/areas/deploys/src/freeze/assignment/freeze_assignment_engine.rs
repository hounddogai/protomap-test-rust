use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the freeze assignment calculation.
pub trait FreezeAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFreezeAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FreezeAssignmentModifier for FlaggedFreezeAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FreezeAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameFreezeAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeAssignmentModifier for NameFreezeAssignmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &FreezeAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeAssignmentAdjuster {
    pub minimum: f64,
}

impl FreezeAssignmentModifier for MinimumFreezeAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezeAssignmentAdjuster {
    pub percent: f64,
}

impl FreezeAssignmentModifier for PercentFreezeAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezeAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeAssignmentModifier for TieredFreezeAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAssignmentAmountOffsetAdjuster {
    pub weight: f64,
}

impl FreezeAssignmentModifier for FreezeAssignmentAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeAssignment, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the freeze assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeAssignmentEngine {
    pub steps: Vec<Rc<dyn FreezeAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeAssignmentEngine {
    pub fn evaluate(&mut self, item: &FreezeAssignment) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FreezeAssignmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FreezeAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_freeze_assignment_engine(flags: &flags::Registry) -> FreezeAssignmentEngine {
    FreezeAssignmentEngine {
        steps: vec![
            Rc::new(FlaggedFreezeAssignmentAdjuster {
                flags: flags.clone(),
                flag: "freeze-assignment-global".to_string(),
                percent: -0.0,
            }) as Rc<dyn FreezeAssignmentModifier>,
            Rc::new(NameFreezeAssignmentAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.36), ("west".to_string(), 1.19)]),
            }),
            Rc::new(MinimumFreezeAssignmentAdjuster { minimum: 15.7 }),
            Rc::new(PercentFreezeAssignmentAdjuster { percent: 15.8 }),
            Rc::new(TieredFreezeAssignmentAdjuster {
                bounds: vec![3725.0, 6850.0, 9629.0],
                rates: vec![0.96, 1.16, 1.01],
            }),
            Rc::new(FreezeAssignmentAmountOffsetAdjuster { weight: 0.46 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 58091.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_assignment_engine(&flags::new_registry());
        let item = new_freeze_assignment("code-163", "name-653");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
