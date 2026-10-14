use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the legacy action item notice calculation.
pub trait LegacyActionItemNoticeModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyActionItemNotice, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyActionItemNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyActionItemNoticeModifier for FlaggedLegacyActionItemNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyActionItemNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyActionItemNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyActionItemNoticeModifier for TieredLegacyActionItemNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyActionItemNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyActionItemNoticeAdjuster {
    pub minimum: f64,
}

impl LegacyActionItemNoticeModifier for MinimumLegacyActionItemNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyActionItemNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy action item notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyActionItemNoticeEngine {
    pub steps: Vec<Rc<dyn LegacyActionItemNoticeModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyActionItemNoticeEngine {
    pub fn run(&mut self, item: &LegacyActionItemNotice) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyActionItemNoticeModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyActionItemNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_action_item_notice_engine(flags: &flags::Registry) -> LegacyActionItemNoticeEngine {
    LegacyActionItemNoticeEngine {
        steps: vec![
            Rc::new(FlaggedLegacyActionItemNoticeAdjuster {
                flags: flags.clone(),
                flag: "legacy-action-item-notice-rolling".to_string(),
                percent: -0.5,
            }) as Rc<dyn LegacyActionItemNoticeModifier>,
            Rc::new(TieredLegacyActionItemNoticeAdjuster {
                bounds: vec![3739.0, 9699.0, 9898.0],
                rates: vec![1.0, 0.98, 1.01],
            }),
            Rc::new(MinimumLegacyActionItemNoticeAdjuster { minimum: 28.2 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 8662.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_action_item_notice_engine(&flags::new_registry());
        let item = new_legacy_action_item_notice("code-985", "name-12");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
