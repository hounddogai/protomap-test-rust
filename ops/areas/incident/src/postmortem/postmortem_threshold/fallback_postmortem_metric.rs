use super::*;

use acme_kit::text;

/// Where a fallback postmortem metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPostmortemMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackPostmortemMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackPostmortemMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback postmortem metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemMetric {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub ttl_seconds: i64,
    pub approved: bool,
    pub duration_minutes: i64,
    pub external_ref: String,
    pub status: FallbackPostmortemMetricStatus,
}

impl FallbackPostmortemMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPostmortemMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPostmortemMetricStatus) -> bool {
        match self.status {
            FallbackPostmortemMetricStatus::Created => {
                match next {
                    FallbackPostmortemMetricStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPostmortemMetricStatus::Validated => {
                return next == FallbackPostmortemMetricStatus::Published;
            }
            FallbackPostmortemMetricStatus::Published => {
                match next {
                    FallbackPostmortemMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPostmortemMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback postmortem metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback postmortem metric", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPostmortemMetric {
        FallbackPostmortemMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            ttl_seconds: self.ttl_seconds,
            approved: self.approved,
            duration_minutes: self.duration_minutes,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback postmortem metric in status created.
pub fn new_fallback_postmortem_metric(code: &str, name: &str) -> FallbackPostmortemMetric {
    FallbackPostmortemMetric {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        ttl_seconds: 0,
        approved: false,
        duration_minutes: 0,
        external_ref: "".to_string(),
        status: FallbackPostmortemMetricStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_postmortem_metric("code-698", "name-945");
        assert_eq!(item.status, FallbackPostmortemMetricStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_fallback_postmortem_metric("code-698", "name-945");
        if item.move_to(FallbackPostmortemMetricStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPostmortemMetricStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_postmortem_metric("code-698", "name-945");
        item.status = FallbackPostmortemMetricStatus::Retired;
        assert!(
            item.move_to(FallbackPostmortemMetricStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
