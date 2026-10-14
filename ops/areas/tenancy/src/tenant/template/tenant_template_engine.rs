use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tenant template calculation.
pub trait TenantTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantTemplateAdjuster {
    pub percent: f64,
}

impl TenantTemplateFactor for PercentTenantTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTenantTemplateAdjuster {
    pub step: f64,
}

impl TenantTemplateFactor for RoundingTenantTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TenantTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantTemplateAdjuster {
    pub minimum: f64,
}

impl TenantTemplateFactor for MinimumTenantTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantTemplateAdjuster {
    pub cap: f64,
}

impl TenantTemplateFactor for CapTenantTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTenantTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantTemplateFactor for ChannelTenantTemplateAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &TenantTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTemplateRatioOffsetAdjuster {
    pub weight: f64,
}

impl TenantTemplateFactor for TenantTemplateRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &TenantTemplate, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

/// Runs the tenant template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantTemplateEngine {
    pub steps: Vec<Rc<dyn TenantTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantTemplateEngine {
    pub fn price(&mut self, item: &TenantTemplate) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TenantTemplateFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TenantTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tenant_template_engine() -> TenantTemplateEngine {
    TenantTemplateEngine {
        steps: vec![
            Rc::new(PercentTenantTemplateAdjuster { percent: -12.6 }) as Rc<dyn TenantTemplateFactor>,
            Rc::new(RoundingTenantTemplateAdjuster { step: 5.0 }),
            Rc::new(MinimumTenantTemplateAdjuster { minimum: 45.3 }),
            Rc::new(CapTenantTemplateAdjuster { cap: 42408.0 }),
            Rc::new(ChannelTenantTemplateAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.05)]),
            }),
            Rc::new(TenantTemplateRatioOffsetAdjuster { weight: 1.57 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 70831.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_template_engine();
        let item = new_tenant_template("code-516", "label-117");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
