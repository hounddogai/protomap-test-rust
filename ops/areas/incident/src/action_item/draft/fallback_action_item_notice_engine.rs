use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback action item notice calculation.
pub trait FallbackActionItemNoticeFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackActionItemNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackActionItemNoticeAdjuster {
    pub percent: f64,
}

impl FallbackActionItemNoticeFactor for PercentFallbackActionItemNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackActionItemNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemNoticeEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl FallbackActionItemNoticeFactor for FallbackActionItemNoticeEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackActionItemNotice, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackActionItemNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackActionItemNoticeFactor for FlaggedFallbackActionItemNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackActionItemNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleFallbackActionItemNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackActionItemNoticeFactor for LocaleFallbackActionItemNoticeAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &FallbackActionItemNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback action item notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackActionItemNoticeEngine {
    pub steps: Vec<Rc<dyn FallbackActionItemNoticeFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackActionItemNoticeEngine {
    pub fn run(&mut self, item: &FallbackActionItemNotice) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FallbackActionItemNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_action_item_notice_engine(flags: &flags::Registry) -> FallbackActionItemNoticeEngine {
    FallbackActionItemNoticeEngine {
        steps: vec![
            Rc::new(PercentFallbackActionItemNoticeAdjuster { percent: 19.2 })
                as Rc<dyn FallbackActionItemNoticeFactor>,
            Rc::new(FallbackActionItemNoticeEndsAtOffsetAdjuster { weight: 1.73 }),
            Rc::new(FlaggedFallbackActionItemNoticeAdjuster {
                flags: flags.clone(),
                flag: "fallback-action-item-notice-capped".to_string(),
                percent: 2.8,
            }),
            Rc::new(LocaleFallbackActionItemNoticeAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.32)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 91999.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_action_item_notice_engine(&flags::new_registry());
        let item = new_fallback_action_item_notice("code-607", "name-857");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
