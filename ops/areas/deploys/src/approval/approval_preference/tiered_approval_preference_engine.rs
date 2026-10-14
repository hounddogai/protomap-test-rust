use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered approval preference calculation.
pub trait TieredApprovalPreferenceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredApprovalPreference, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredApprovalPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredApprovalPreferenceModifier for FlaggedTieredApprovalPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredApprovalPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredApprovalPreferenceAdjuster {
    pub cap: f64,
}

impl TieredApprovalPreferenceModifier for CapTieredApprovalPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredApprovalPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredApprovalPreferenceAdjuster {
    pub step: f64,
}

impl TieredApprovalPreferenceModifier for RoundingTieredApprovalPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredApprovalPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalPreferenceUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl TieredApprovalPreferenceModifier for TieredApprovalPreferenceUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &TieredApprovalPreference, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTieredApprovalPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredApprovalPreferenceModifier for ChannelTieredApprovalPreferenceAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &TieredApprovalPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredApprovalPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredApprovalPreferenceModifier for TieredTieredApprovalPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredApprovalPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered approval preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredApprovalPreferenceEngine {
    pub steps: Vec<Rc<dyn TieredApprovalPreferenceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredApprovalPreferenceEngine {
    pub fn evaluate(&mut self, item: &TieredApprovalPreference) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[TieredApprovalPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_approval_preference_engine(flags: &flags::Registry) -> TieredApprovalPreferenceEngine {
    TieredApprovalPreferenceEngine {
        steps: vec![
            Rc::new(FlaggedTieredApprovalPreferenceAdjuster {
                flags: flags.clone(),
                flag: "tiered-approval-preference-adaptive".to_string(),
                percent: -6.3,
            }) as Rc<dyn TieredApprovalPreferenceModifier>,
            Rc::new(CapTieredApprovalPreferenceAdjuster { cap: 8235.0 }),
            Rc::new(RoundingTieredApprovalPreferenceAdjuster { step: 0.25 }),
            Rc::new(TieredApprovalPreferenceUpdatedAtOffsetAdjuster { weight: 0.16 }),
            Rc::new(ChannelTieredApprovalPreferenceAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.31),
                    ("online".to_string(), 1.15),
                    ("retail".to_string(), 0.79),
                ]),
            }),
            Rc::new(TieredTieredApprovalPreferenceAdjuster {
                bounds: vec![2070.0, 6130.0, 8661.0],
                rates: vec![1.0, 1.1, 0.84],
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 39319.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_approval_preference_engine(&flags::new_registry());
        let item = new_tiered_approval_preference("code-135", "title-643");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
