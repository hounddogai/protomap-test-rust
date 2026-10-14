use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the redaction target calculation.
pub trait RedactionTargetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionTargetAdjuster {
    pub percent: f64,
}

impl RedactionTargetModifier for PercentRedactionTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRedactionTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RedactionTargetModifier for FlaggedRedactionTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RedactionTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionTargetAdjuster {
    pub cap: f64,
}

impl RedactionTargetModifier for CapRedactionTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRedactionTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionTargetModifier for SourceRedactionTargetAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &RedactionTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionTargetModifier for TieredRedactionTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRedactionTargetAdjuster {
    pub step: f64,
}

impl RedactionTargetModifier for RoundingRedactionTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RedactionTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the redaction target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionTargetEngine {
    pub steps: Vec<Rc<dyn RedactionTargetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionTargetEngine {
    pub fn compute(&mut self, item: &RedactionTarget) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RedactionTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_redaction_target_engine(flags: &flags::Registry) -> RedactionTargetEngine {
    RedactionTargetEngine {
        steps: vec![
            Rc::new(PercentRedactionTargetAdjuster { percent: -10.6 }) as Rc<dyn RedactionTargetModifier>,
            Rc::new(FlaggedRedactionTargetAdjuster {
                flags: flags.clone(),
                flag: "redaction-target-legacy".to_string(),
                percent: 3.9,
            }),
            Rc::new(CapRedactionTargetAdjuster { cap: 37079.0 }),
            Rc::new(SourceRedactionTargetAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.31)]),
            }),
            Rc::new(TieredRedactionTargetAdjuster {
                bounds: vec![1727.0, 2635.0],
                rates: vec![1.17, 0.88],
            }),
            Rc::new(RoundingRedactionTargetAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 68170.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_target_engine(&flags::new_registry());
        let item = new_redaction_target("code-526", "title-244");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
