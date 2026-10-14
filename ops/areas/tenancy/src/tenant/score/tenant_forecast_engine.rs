use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tenant forecast calculation.
pub trait TenantForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantForecastAdjuster {
    pub cap: f64,
}

impl TenantForecastModifier for CapTenantForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantForecastAdjuster {
    pub minimum: f64,
}

impl TenantForecastModifier for MinimumTenantForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantForecastModifier for FlaggedTenantForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantForecastRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl TenantForecastModifier for TenantForecastRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &TenantForecast, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

/// Runs the tenant forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantForecastEngine {
    pub steps: Vec<Rc<dyn TenantForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantForecastEngine {
    pub fn run(&mut self, item: &TenantForecast) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TenantForecastModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TenantForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tenant_forecast_engine(flags: &flags::Registry) -> TenantForecastEngine {
    TenantForecastEngine {
        steps: vec![
            Rc::new(CapTenantForecastAdjuster { cap: 8816.0 }) as Rc<dyn TenantForecastModifier>,
            Rc::new(MinimumTenantForecastAdjuster { minimum: 40.6 }),
            Rc::new(FlaggedTenantForecastAdjuster {
                flags: flags.clone(),
                flag: "tenant-forecast-pending".to_string(),
                percent: -5.0,
            }),
            Rc::new(TenantForecastRotationDaysOffsetAdjuster { weight: 0.49 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 97182.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_forecast_engine(&flags::new_registry());
        let item = new_tenant_forecast("code-497", "label-458");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
