use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback dashboard draft calculation.
pub trait FallbackDashboardDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackDashboardDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackDashboardDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackDashboardDraftStep for TieredFallbackDashboardDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackDashboardDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardDraftMarginOffsetAdjuster {
    pub weight: f64,
}

impl FallbackDashboardDraftStep for FallbackDashboardDraftMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackDashboardDraft, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackDashboardDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackDashboardDraftStep for FlaggedFallbackDashboardDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackDashboardDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackDashboardDraftAdjuster {
    pub step: f64,
}

impl FallbackDashboardDraftStep for RoundingFallbackDashboardDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackDashboardDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackDashboardDraftAdjuster {
    pub cap: f64,
}

impl FallbackDashboardDraftStep for CapFallbackDashboardDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackDashboardDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFallbackDashboardDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackDashboardDraftStep for LabelFallbackDashboardDraftAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FallbackDashboardDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback dashboard draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackDashboardDraftEngine {
    pub steps: Vec<Rc<dyn FallbackDashboardDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackDashboardDraftEngine {
    pub fn run(&mut self, item: &FallbackDashboardDraft) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[FallbackDashboardDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_dashboard_draft_engine(flags: &flags::Registry) -> FallbackDashboardDraftEngine {
    FallbackDashboardDraftEngine {
        steps: vec![
            Rc::new(TieredFallbackDashboardDraftAdjuster {
                bounds: vec![5055.0, 9287.0],
                rates: vec![1.09, 1.01],
            }) as Rc<dyn FallbackDashboardDraftStep>,
            Rc::new(FallbackDashboardDraftMarginOffsetAdjuster { weight: 1.89 }),
            Rc::new(FlaggedFallbackDashboardDraftAdjuster {
                flags: flags.clone(),
                flag: "fallback-dashboard-draft-fallback".to_string(),
                percent: -7.3,
            }),
            Rc::new(RoundingFallbackDashboardDraftAdjuster { step: 0.5 }),
            Rc::new(CapFallbackDashboardDraftAdjuster { cap: 45735.0 }),
            Rc::new(LabelFallbackDashboardDraftAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.96),
                    ("south".to_string(), 0.75),
                    ("online".to_string(), 1.1),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 13031.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_dashboard_draft_engine(&flags::new_registry());
        let item = new_fallback_dashboard_draft("code-904", "label-707");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
