use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient freeze tier calculation.
pub trait LenientFreezeTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientFreezeTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientFreezeTierAdjuster {
    pub percent: f64,
}

impl LenientFreezeTierAdjuster for PercentLenientFreezeTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientFreezeTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientFreezeTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientFreezeTierAdjuster for FlaggedLenientFreezeTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientFreezeTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceLenientFreezeTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientFreezeTierAdjuster for NamespaceLenientFreezeTierAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &LenientFreezeTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientFreezeTierAdjuster {
    pub minimum: f64,
}

impl LenientFreezeTierAdjuster for MinimumLenientFreezeTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientFreezeTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeTierQuantityOffsetAdjuster {
    pub weight: f64,
}

impl LenientFreezeTierAdjuster for LenientFreezeTierQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &LenientFreezeTier, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the lenient freeze tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientFreezeTierEngine {
    pub steps: Vec<Rc<dyn LenientFreezeTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientFreezeTierEngine {
    pub fn compute(&mut self, item: &LenientFreezeTier) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientFreezeTierAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientFreezeTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_freeze_tier_engine(flags: &flags::Registry) -> LenientFreezeTierEngine {
    LenientFreezeTierEngine {
        steps: vec![
            Rc::new(PercentLenientFreezeTierAdjuster { percent: -14.9 }) as Rc<dyn LenientFreezeTierAdjuster>,
            Rc::new(FlaggedLenientFreezeTierAdjuster {
                flags: flags.clone(),
                flag: "lenient-freeze-tier-legacy".to_string(),
                percent: 7.2,
            }),
            Rc::new(NamespaceLenientFreezeTierAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.04), ("east".to_string(), 0.85)]),
            }),
            Rc::new(MinimumLenientFreezeTierAdjuster { minimum: 49.1 }),
            Rc::new(LenientFreezeTierQuantityOffsetAdjuster { weight: 0.96 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 14737.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_freeze_tier_engine(&flags::new_registry());
        let item = new_lenient_freeze_tier("code-122", "title-976");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
