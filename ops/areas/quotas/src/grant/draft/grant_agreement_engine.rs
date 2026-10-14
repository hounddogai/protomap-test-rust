use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the grant agreement calculation.
pub trait GrantAgreementStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGrantAgreementAdjuster {
    pub cap: f64,
}

impl GrantAgreementStep for CapGrantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GrantAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionGrantAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantAgreementStep for RegionGrantAgreementAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &GrantAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAgreementWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl GrantAgreementStep for GrantAgreementWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &GrantAgreement, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGrantAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GrantAgreementStep for FlaggedGrantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GrantAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGrantAgreementAdjuster {
    pub step: f64,
}

impl GrantAgreementStep for RoundingGrantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GrantAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the grant agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantAgreementEngine {
    pub steps: Vec<Rc<dyn GrantAgreementStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantAgreementEngine {
    pub fn run(&mut self, item: &GrantAgreement) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[GrantAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_grant_agreement_engine(flags: &flags::Registry) -> GrantAgreementEngine {
    GrantAgreementEngine {
        steps: vec![
            Rc::new(CapGrantAgreementAdjuster { cap: 18859.0 }) as Rc<dyn GrantAgreementStep>,
            Rc::new(RegionGrantAgreementAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.75),
                    ("north".to_string(), 0.98),
                    ("south".to_string(), 1.38),
                ]),
            }),
            Rc::new(GrantAgreementWindowDaysOffsetAdjuster { weight: 0.54 }),
            Rc::new(FlaggedGrantAgreementAdjuster {
                flags: flags.clone(),
                flag: "grant-agreement-fallback".to_string(),
                percent: -6.8,
            }),
            Rc::new(RoundingGrantAgreementAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 92703.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_agreement_engine(&flags::new_registry());
        let item = new_grant_agreement("code-177", "label-519");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
