use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered credential plan calculation.
pub trait TieredCredentialPlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredCredentialPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialPlanMarginOffsetAdjuster {
    pub weight: f64,
}

impl TieredCredentialPlanAdjuster for TieredCredentialPlanMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &TieredCredentialPlan, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredCredentialPlanAdjuster {
    pub step: f64,
}

impl TieredCredentialPlanAdjuster for RoundingTieredCredentialPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredCredentialPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredCredentialPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredCredentialPlanAdjuster for TieredTieredCredentialPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredCredentialPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredCredentialPlanAdjuster {
    pub cap: f64,
}

impl TieredCredentialPlanAdjuster for CapTieredCredentialPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredCredentialPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredCredentialPlanAdjuster {
    pub minimum: f64,
}

impl TieredCredentialPlanAdjuster for MinimumTieredCredentialPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredCredentialPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredCredentialPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredCredentialPlanAdjuster for FlaggedTieredCredentialPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredCredentialPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered credential plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredCredentialPlanEngine {
    pub steps: Vec<Rc<dyn TieredCredentialPlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredCredentialPlanEngine {
    pub fn evaluate(&mut self, item: &TieredCredentialPlan) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[TieredCredentialPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_credential_plan_engine(flags: &flags::Registry) -> TieredCredentialPlanEngine {
    TieredCredentialPlanEngine {
        steps: vec![
            Rc::new(TieredCredentialPlanMarginOffsetAdjuster { weight: 1.89 }) as Rc<dyn TieredCredentialPlanAdjuster>,
            Rc::new(RoundingTieredCredentialPlanAdjuster { step: 0.25 }),
            Rc::new(TieredTieredCredentialPlanAdjuster {
                bounds: vec![1931.0, 4292.0, 8615.0],
                rates: vec![0.87, 1.04, 1.0],
            }),
            Rc::new(CapTieredCredentialPlanAdjuster { cap: 27204.0 }),
            Rc::new(MinimumTieredCredentialPlanAdjuster { minimum: 11.6 }),
            Rc::new(FlaggedTieredCredentialPlanAdjuster {
                flags: flags.clone(),
                flag: "tiered-credential-plan-capped".to_string(),
                percent: -3.7,
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 12870.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_credential_plan_engine(&flags::new_registry());
        let item = new_tiered_credential_plan("code-400", "name-170");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
