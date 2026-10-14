use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the trust domain template calculation.
pub trait TrustDomainTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainTemplateFactor for FlaggedTrustDomainTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTrustDomainTemplateAdjuster {
    pub minimum: f64,
}

impl TrustDomainTemplateFactor for MinimumTrustDomainTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TrustDomainTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainTemplateAdjuster {
    pub step: f64,
}

impl TrustDomainTemplateFactor for RoundingTrustDomainTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the trust domain template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainTemplateEngine {
    pub steps: Vec<Rc<dyn TrustDomainTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainTemplateEngine {
    pub fn price(&mut self, item: &TrustDomainTemplate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_trust_domain_template_engine(flags: &flags::Registry) -> TrustDomainTemplateEngine {
    TrustDomainTemplateEngine {
        steps: vec![
            Rc::new(FlaggedTrustDomainTemplateAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-template-legacy".to_string(),
                percent: 9.6,
            }) as Rc<dyn TrustDomainTemplateFactor>,
            Rc::new(MinimumTrustDomainTemplateAdjuster { minimum: 34.9 }),
            Rc::new(RoundingTrustDomainTemplateAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 76721.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_template_engine(&flags::new_registry());
        let item = new_trust_domain_template("code-668", "label-478");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
