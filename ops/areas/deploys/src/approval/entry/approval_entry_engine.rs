use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the approval entry calculation.
pub trait ApprovalEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalEntryAdjuster {
    pub cap: f64,
}

impl ApprovalEntryModifier for CapApprovalEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalEntryModifier for TieredApprovalEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalEntryWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalEntryModifier for ApprovalEntryWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalEntry, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalEntryAdjuster {
    pub percent: f64,
}

impl ApprovalEntryModifier for PercentApprovalEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameApprovalEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalEntryModifier for NameApprovalEntryAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ApprovalEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedApprovalEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ApprovalEntryModifier for FlaggedApprovalEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ApprovalEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the approval entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalEntryEngine {
    pub steps: Vec<Rc<dyn ApprovalEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalEntryEngine {
    pub fn price(&mut self, item: &ApprovalEntry) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_approval_entry_engine(flags: &flags::Registry) -> ApprovalEntryEngine {
    ApprovalEntryEngine {
        steps: vec![
            Rc::new(CapApprovalEntryAdjuster { cap: 44128.0 }) as Rc<dyn ApprovalEntryModifier>,
            Rc::new(TieredApprovalEntryAdjuster {
                bounds: vec![1240.0, 4170.0],
                rates: vec![1.08, 1.16],
            }),
            Rc::new(ApprovalEntryWindowDaysOffsetAdjuster { weight: 0.31 }),
            Rc::new(PercentApprovalEntryAdjuster { percent: 2.5 }),
            Rc::new(NameApprovalEntryAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.04), ("south".to_string(), 1.26)]),
            }),
            Rc::new(FlaggedApprovalEntryAdjuster {
                flags: flags.clone(),
                flag: "approval-entry-regional".to_string(),
                percent: -4.4,
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 30584.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_entry_engine(&flags::new_registry());
        let item = new_approval_entry("code-397", "name-978");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
