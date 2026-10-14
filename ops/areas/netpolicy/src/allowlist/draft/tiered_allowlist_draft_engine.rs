use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered allowlist draft calculation.
pub trait TieredAllowlistDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredAllowlistDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredAllowlistDraftAdjuster {
    pub step: f64,
}

impl TieredAllowlistDraftStep for RoundingTieredAllowlistDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredAllowlistDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredAllowlistDraftAdjuster {
    pub cap: f64,
}

impl TieredAllowlistDraftStep for CapTieredAllowlistDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredAllowlistDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTieredAllowlistDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredAllowlistDraftStep for ChannelTieredAllowlistDraftAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &TieredAllowlistDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredAllowlistDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredAllowlistDraftStep for TieredTieredAllowlistDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredAllowlistDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered allowlist draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredAllowlistDraftEngine {
    pub steps: Vec<Rc<dyn TieredAllowlistDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredAllowlistDraftEngine {
    pub fn compute(&mut self, item: &TieredAllowlistDraft) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[TieredAllowlistDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_allowlist_draft_engine() -> TieredAllowlistDraftEngine {
    TieredAllowlistDraftEngine {
        steps: vec![
            Rc::new(RoundingTieredAllowlistDraftAdjuster { step: 0.1 }) as Rc<dyn TieredAllowlistDraftStep>,
            Rc::new(CapTieredAllowlistDraftAdjuster { cap: 25252.0 }),
            Rc::new(ChannelTieredAllowlistDraftAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.08)]),
            }),
            Rc::new(TieredTieredAllowlistDraftAdjuster {
                bounds: vec![540.0, 3637.0, 4065.0, 8935.0],
                rates: vec![0.94, 0.91, 1.02, 1.19],
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 9346.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_allowlist_draft_engine();
        let item = new_tiered_allowlist_draft("code-173", "name-912");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
