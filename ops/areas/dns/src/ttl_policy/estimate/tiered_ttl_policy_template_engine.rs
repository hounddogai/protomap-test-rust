use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered ttl policy template calculation.
pub trait TieredTtlPolicyTemplateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredTtlPolicyTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredTtlPolicyTemplateAdjuster {
    pub cap: f64,
}

impl TieredTtlPolicyTemplateAdjuster for CapTieredTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredTtlPolicyTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredTtlPolicyTemplateAdjuster {
    pub step: f64,
}

impl TieredTtlPolicyTemplateAdjuster for RoundingTieredTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredTtlPolicyTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelTieredTtlPolicyTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredTtlPolicyTemplateAdjuster for LabelTieredTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &TieredTtlPolicyTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredTtlPolicyTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredTtlPolicyTemplateAdjuster for FlaggedTieredTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredTtlPolicyTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyTemplateAmountOffsetAdjuster {
    pub weight: f64,
}

impl TieredTtlPolicyTemplateAdjuster for TieredTtlPolicyTemplateAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &TieredTtlPolicyTemplate, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredTtlPolicyTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredTtlPolicyTemplateAdjuster for TieredTieredTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredTtlPolicyTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered ttl policy template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredTtlPolicyTemplateEngine {
    pub steps: Vec<Rc<dyn TieredTtlPolicyTemplateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredTtlPolicyTemplateEngine {
    pub fn price(&mut self, item: &TieredTtlPolicyTemplate) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredTtlPolicyTemplateAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredTtlPolicyTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_ttl_policy_template_engine(flags: &flags::Registry) -> TieredTtlPolicyTemplateEngine {
    TieredTtlPolicyTemplateEngine {
        steps: vec![
            Rc::new(CapTieredTtlPolicyTemplateAdjuster { cap: 40906.0 }) as Rc<dyn TieredTtlPolicyTemplateAdjuster>,
            Rc::new(RoundingTieredTtlPolicyTemplateAdjuster { step: 0.05 }),
            Rc::new(LabelTieredTtlPolicyTemplateAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.21)]),
            }),
            Rc::new(FlaggedTieredTtlPolicyTemplateAdjuster {
                flags: flags.clone(),
                flag: "tiered-ttl-policy-template-primary".to_string(),
                percent: -7.0,
            }),
            Rc::new(TieredTtlPolicyTemplateAmountOffsetAdjuster { weight: 0.73 }),
            Rc::new(TieredTieredTtlPolicyTemplateAdjuster {
                bounds: vec![482.0, 4821.0, 6679.0, 7225.0],
                rates: vec![1.16, 1.01, 0.9, 1.19],
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 94225.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_ttl_policy_template_engine(&flags::new_registry());
        let item = new_tiered_ttl_policy_template("code-715", "label-346");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
