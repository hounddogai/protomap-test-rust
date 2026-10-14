use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual rollback rollup calculation.
pub trait ManualRollbackRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualRollbackRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualRollbackRollupAdjuster {
    pub cap: f64,
}

impl ManualRollbackRollupFactor for CapManualRollbackRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualRollbackRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualRollbackRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualRollbackRollupFactor for FlaggedManualRollbackRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualRollbackRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualRollbackRollupAdjuster {
    pub minimum: f64,
}

impl ManualRollbackRollupFactor for MinimumManualRollbackRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualRollbackRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual rollback rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualRollbackRollupEngine {
    pub steps: Vec<Rc<dyn ManualRollbackRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualRollbackRollupEngine {
    pub fn price(&mut self, item: &ManualRollbackRollup) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualRollbackRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualRollbackRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_rollback_rollup_engine(flags: &flags::Registry) -> ManualRollbackRollupEngine {
    ManualRollbackRollupEngine {
        steps: vec![
            Rc::new(CapManualRollbackRollupAdjuster { cap: 36189.0 }) as Rc<dyn ManualRollbackRollupFactor>,
            Rc::new(FlaggedManualRollbackRollupAdjuster {
                flags: flags.clone(),
                flag: "manual-rollback-rollup-adaptive".to_string(),
                percent: -4.7,
            }),
            Rc::new(MinimumManualRollbackRollupAdjuster { minimum: 1.3 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 56016.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_rollback_rollup_engine(&flags::new_registry());
        let item = new_manual_rollback_rollup("code-426", "name-9");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
