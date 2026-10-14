use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered freeze group calculation.
pub trait TieredFreezeGroupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredFreezeGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredFreezeGroupAdjuster {
    pub step: f64,
}

impl TieredFreezeGroupModifier for RoundingTieredFreezeGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredFreezeGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredFreezeGroupAdjuster {
    pub cap: f64,
}

impl TieredFreezeGroupModifier for CapTieredFreezeGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredFreezeGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredFreezeGroupAdjuster {
    pub minimum: f64,
}

impl TieredFreezeGroupModifier for MinimumTieredFreezeGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredFreezeGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredFreezeGroupAdjuster {
    pub percent: f64,
}

impl TieredFreezeGroupModifier for PercentTieredFreezeGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredFreezeGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered freeze group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredFreezeGroupEngine {
    pub steps: Vec<Rc<dyn TieredFreezeGroupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredFreezeGroupEngine {
    pub fn price(&mut self, item: &TieredFreezeGroup) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredFreezeGroupModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredFreezeGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_freeze_group_engine() -> TieredFreezeGroupEngine {
    TieredFreezeGroupEngine {
        steps: vec![
            Rc::new(RoundingTieredFreezeGroupAdjuster { step: 1.0 }) as Rc<dyn TieredFreezeGroupModifier>,
            Rc::new(CapTieredFreezeGroupAdjuster { cap: 24617.0 }),
            Rc::new(MinimumTieredFreezeGroupAdjuster { minimum: 36.3 }),
            Rc::new(PercentTieredFreezeGroupAdjuster { percent: -22.8 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 1264.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_freeze_group_engine();
        let item = new_tiered_freeze_group("code-772", "name-395");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
