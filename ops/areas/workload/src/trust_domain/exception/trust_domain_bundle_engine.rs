use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the trust domain bundle calculation.
pub trait TrustDomainBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainBundleAdjuster {
    pub step: f64,
}

impl TrustDomainBundleModifier for RoundingTrustDomainBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTrustDomainBundleAdjuster {
    pub minimum: f64,
}

impl TrustDomainBundleModifier for MinimumTrustDomainBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TrustDomainBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainBundleModifier for FlaggedTrustDomainBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the trust domain bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainBundleEngine {
    pub steps: Vec<Rc<dyn TrustDomainBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainBundleEngine {
    pub fn run(&mut self, item: &TrustDomainBundle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_trust_domain_bundle_engine(flags: &flags::Registry) -> TrustDomainBundleEngine {
    TrustDomainBundleEngine {
        steps: vec![
            Rc::new(RoundingTrustDomainBundleAdjuster { step: 0.05 }) as Rc<dyn TrustDomainBundleModifier>,
            Rc::new(MinimumTrustDomainBundleAdjuster { minimum: 11.1 }),
            Rc::new(FlaggedTrustDomainBundleAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-bundle-legacy".to_string(),
                percent: -7.1,
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 54895.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_bundle_engine(&flags::new_registry());
        let item = new_trust_domain_bundle("code-843", "label-986");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
