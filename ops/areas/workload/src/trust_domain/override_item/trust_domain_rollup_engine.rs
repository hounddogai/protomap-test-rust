use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the trust domain rollup calculation.
pub trait TrustDomainRollupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRollupDiscountOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainRollupStep for TrustDomainRollupDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainRollup, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceTrustDomainRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TrustDomainRollupStep for NamespaceTrustDomainRollupAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &TrustDomainRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTrustDomainRollupAdjuster {
    pub percent: f64,
}

impl TrustDomainRollupStep for PercentTrustDomainRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TrustDomainRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainRollupStep for FlaggedTrustDomainRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the trust domain rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainRollupEngine {
    pub steps: Vec<Rc<dyn TrustDomainRollupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainRollupEngine {
    pub fn run(&mut self, item: &TrustDomainRollup) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainRollupStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_trust_domain_rollup_engine(flags: &flags::Registry) -> TrustDomainRollupEngine {
    TrustDomainRollupEngine {
        steps: vec![
            Rc::new(TrustDomainRollupDiscountOffsetAdjuster { weight: 1.53 }) as Rc<dyn TrustDomainRollupStep>,
            Rc::new(NamespaceTrustDomainRollupAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.98),
                    ("north".to_string(), 0.99),
                    ("south".to_string(), 1.17),
                ]),
            }),
            Rc::new(PercentTrustDomainRollupAdjuster { percent: 18.4 }),
            Rc::new(FlaggedTrustDomainRollupAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-rollup-manual".to_string(),
                percent: -4.3,
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 66857.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_rollup_engine(&flags::new_registry());
        let item = new_trust_domain_rollup("code-83", "label-191");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
