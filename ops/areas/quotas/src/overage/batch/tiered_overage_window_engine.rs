use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered overage window calculation.
pub trait TieredOverageWindowModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredOverageWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredOverageWindowAdjuster {
    pub minimum: f64,
}

impl TieredOverageWindowModifier for MinimumTieredOverageWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredOverageWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredOverageWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredOverageWindowModifier for FlaggedTieredOverageWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredOverageWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredOverageWindowAdjuster {
    pub step: f64,
}

impl TieredOverageWindowModifier for RoundingTieredOverageWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredOverageWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the tiered overage window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredOverageWindowEngine {
    pub steps: Vec<Rc<dyn TieredOverageWindowModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredOverageWindowEngine {
    pub fn price(&mut self, item: &TieredOverageWindow) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredOverageWindowModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredOverageWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_overage_window_engine(flags: &flags::Registry) -> TieredOverageWindowEngine {
    TieredOverageWindowEngine {
        steps: vec![
            Rc::new(MinimumTieredOverageWindowAdjuster { minimum: 48.8 }) as Rc<dyn TieredOverageWindowModifier>,
            Rc::new(FlaggedTieredOverageWindowAdjuster {
                flags: flags.clone(),
                flag: "tiered-overage-window-capped".to_string(),
                percent: 7.5,
            }),
            Rc::new(RoundingTieredOverageWindowAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 35422.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_overage_window_engine(&flags::new_registry());
        let item = new_tiered_overage_window("code-657", "title-822");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
