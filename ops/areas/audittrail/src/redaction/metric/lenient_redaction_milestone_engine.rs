use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient redaction milestone calculation.
pub trait LenientRedactionMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientRedactionMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientRedactionMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientRedactionMilestoneStep for TieredLenientRedactionMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientRedactionMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLenientRedactionMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientRedactionMilestoneStep for NameLenientRedactionMilestoneAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LenientRedactionMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientRedactionMilestoneAdjuster {
    pub minimum: f64,
}

impl LenientRedactionMilestoneStep for MinimumLenientRedactionMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientRedactionMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionMilestoneLimitOffsetAdjuster {
    pub weight: f64,
}

impl LenientRedactionMilestoneStep for LenientRedactionMilestoneLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &LenientRedactionMilestone, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientRedactionMilestoneAdjuster {
    pub cap: f64,
}

impl LenientRedactionMilestoneStep for CapLenientRedactionMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientRedactionMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientRedactionMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientRedactionMilestoneStep for FlaggedLenientRedactionMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientRedactionMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient redaction milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientRedactionMilestoneEngine {
    pub steps: Vec<Rc<dyn LenientRedactionMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientRedactionMilestoneEngine {
    pub fn compute(&mut self, item: &LenientRedactionMilestone) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[LenientRedactionMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_redaction_milestone_engine(flags: &flags::Registry) -> LenientRedactionMilestoneEngine {
    LenientRedactionMilestoneEngine {
        steps: vec![
            Rc::new(TieredLenientRedactionMilestoneAdjuster {
                bounds: vec![558.0, 1644.0, 8609.0],
                rates: vec![1.19, 0.92, 1.07],
            }) as Rc<dyn LenientRedactionMilestoneStep>,
            Rc::new(NameLenientRedactionMilestoneAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.97)]),
            }),
            Rc::new(MinimumLenientRedactionMilestoneAdjuster { minimum: 0.2 }),
            Rc::new(LenientRedactionMilestoneLimitOffsetAdjuster { weight: 1.1 }),
            Rc::new(CapLenientRedactionMilestoneAdjuster { cap: 8883.0 }),
            Rc::new(FlaggedLenientRedactionMilestoneAdjuster {
                flags: flags.clone(),
                flag: "lenient-redaction-milestone-global".to_string(),
                percent: -4.2,
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 35482.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_redaction_milestone_engine(&flags::new_registry());
        let item = new_lenient_redaction_milestone("code-79", "name-614");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
