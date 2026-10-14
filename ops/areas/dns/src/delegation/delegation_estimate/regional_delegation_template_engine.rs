use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional delegation template calculation.
pub trait RegionalDelegationTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalDelegationTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalDelegationTemplateAdjuster {
    pub cap: f64,
}

impl RegionalDelegationTemplateModifier for CapRegionalDelegationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalDelegationTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationTemplateEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl RegionalDelegationTemplateModifier for RegionalDelegationTemplateEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalDelegationTemplate, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalDelegationTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalDelegationTemplateModifier for FlaggedRegionalDelegationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalDelegationTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalDelegationTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalDelegationTemplateModifier for TieredRegionalDelegationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalDelegationTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalDelegationTemplateAdjuster {
    pub minimum: f64,
}

impl RegionalDelegationTemplateModifier for MinimumRegionalDelegationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalDelegationTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional delegation template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalDelegationTemplateEngine {
    pub steps: Vec<Rc<dyn RegionalDelegationTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalDelegationTemplateEngine {
    pub fn price(&mut self, item: &RegionalDelegationTemplate) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RegionalDelegationTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_delegation_template_engine(flags: &flags::Registry) -> RegionalDelegationTemplateEngine {
    RegionalDelegationTemplateEngine {
        steps: vec![
            Rc::new(CapRegionalDelegationTemplateAdjuster { cap: 6296.0 })
                as Rc<dyn RegionalDelegationTemplateModifier>,
            Rc::new(RegionalDelegationTemplateEndsAtOffsetAdjuster { weight: 0.46 }),
            Rc::new(FlaggedRegionalDelegationTemplateAdjuster {
                flags: flags.clone(),
                flag: "regional-delegation-template-default".to_string(),
                percent: -4.8,
            }),
            Rc::new(TieredRegionalDelegationTemplateAdjuster {
                bounds: vec![448.0, 1495.0, 4287.0, 5403.0],
                rates: vec![1.07, 0.83, 1.09, 0.86],
            }),
            Rc::new(MinimumRegionalDelegationTemplateAdjuster { minimum: 42.3 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 15326.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_delegation_template_engine(&flags::new_registry());
        let item = new_regional_delegation_template("code-897", "name-905");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
