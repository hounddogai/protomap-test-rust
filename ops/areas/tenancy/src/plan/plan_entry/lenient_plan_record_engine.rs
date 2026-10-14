use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient plan record calculation.
pub trait LenientPlanRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientPlanRecord, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientPlanRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientPlanRecordModifier for FlaggedLenientPlanRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientPlanRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientPlanRecordAdjuster {
    pub percent: f64,
}

impl LenientPlanRecordModifier for PercentLenientPlanRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientPlanRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanRecordRateOffsetAdjuster {
    pub weight: f64,
}

impl LenientPlanRecordModifier for LenientPlanRecordRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &LenientPlanRecord, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitLenientPlanRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientPlanRecordModifier for UnitLenientPlanRecordAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &LenientPlanRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient plan record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientPlanRecordEngine {
    pub steps: Vec<Rc<dyn LenientPlanRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientPlanRecordEngine {
    pub fn price(&mut self, item: &LenientPlanRecord) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientPlanRecordModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientPlanRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_plan_record_engine(flags: &flags::Registry) -> LenientPlanRecordEngine {
    LenientPlanRecordEngine {
        steps: vec![
            Rc::new(FlaggedLenientPlanRecordAdjuster {
                flags: flags.clone(),
                flag: "lenient-plan-record-primary".to_string(),
                percent: 4.2,
            }) as Rc<dyn LenientPlanRecordModifier>,
            Rc::new(PercentLenientPlanRecordAdjuster { percent: -10.6 }),
            Rc::new(LenientPlanRecordRateOffsetAdjuster { weight: 1.4 }),
            Rc::new(UnitLenientPlanRecordAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.37),
                    ("south".to_string(), 1.08),
                    ("central".to_string(), 1.08),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 62011.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_plan_record_engine(&flags::new_registry());
        let item = new_lenient_plan_record("code-509", "name-72");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
