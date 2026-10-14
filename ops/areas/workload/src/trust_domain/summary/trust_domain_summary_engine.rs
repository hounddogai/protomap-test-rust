use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the trust domain summary calculation.
pub trait TrustDomainSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainSummary, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainSummaryModifier for FlaggedTrustDomainSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleTrustDomainSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TrustDomainSummaryModifier for TitleTrustDomainSummaryAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &TrustDomainSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainSummaryAdjuster {
    pub step: f64,
}

impl TrustDomainSummaryModifier for RoundingTrustDomainSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTrustDomainSummaryAdjuster {
    pub cap: f64,
}

impl TrustDomainSummaryModifier for CapTrustDomainSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TrustDomainSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the trust domain summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainSummaryEngine {
    pub steps: Vec<Rc<dyn TrustDomainSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainSummaryEngine {
    pub fn run(&mut self, item: &TrustDomainSummary) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainSummaryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_trust_domain_summary_engine(flags: &flags::Registry) -> TrustDomainSummaryEngine {
    TrustDomainSummaryEngine {
        steps: vec![
            Rc::new(FlaggedTrustDomainSummaryAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-summary-strict".to_string(),
                percent: 8.6,
            }) as Rc<dyn TrustDomainSummaryModifier>,
            Rc::new(TitleTrustDomainSummaryAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.93),
                    ("east".to_string(), 1.08),
                    ("south".to_string(), 0.91),
                ]),
            }),
            Rc::new(RoundingTrustDomainSummaryAdjuster { step: 0.5 }),
            Rc::new(CapTrustDomainSummaryAdjuster { cap: 27909.0 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 48160.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_summary_engine(&flags::new_registry());
        let item = new_trust_domain_summary("code-485", "title-997");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
