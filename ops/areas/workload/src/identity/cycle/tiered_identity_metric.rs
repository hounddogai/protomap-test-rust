use super::*;

use acme_kit::text;

/// Where a tiered identity metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredIdentityMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredIdentityMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for TieredIdentityMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered identity metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityMetric {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub tenant_id: String,
    pub reviewer: String,
    pub segment: String,
    pub country: String,
    pub owner_id: String,
    pub source: String,
    pub principal: String,
    pub ratio: f64,
    pub status: TieredIdentityMetricStatus,
}

impl TieredIdentityMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredIdentityMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredIdentityMetricStatus) -> bool {
        match self.status {
            TieredIdentityMetricStatus::Idle => {
                return next == TieredIdentityMetricStatus::Draining || next == TieredIdentityMetricStatus::Stopped;
            }
            TieredIdentityMetricStatus::Warming => {
                match next {
                    TieredIdentityMetricStatus::Draining | TieredIdentityMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredIdentityMetricStatus::Ready => {
                match next {
                    TieredIdentityMetricStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredIdentityMetricStatus::Draining => {
                return next == TieredIdentityMetricStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredIdentityMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered identity metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ratio
    }

    pub fn copy(&self) -> TieredIdentityMetric {
        TieredIdentityMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            tenant_id: self.tenant_id.clone(),
            reviewer: self.reviewer.clone(),
            segment: self.segment.clone(),
            country: self.country.clone(),
            owner_id: self.owner_id.clone(),
            source: self.source.clone(),
            principal: self.principal.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a tiered identity metric in status idle.
pub fn new_tiered_identity_metric(code: &str, name: &str) -> TieredIdentityMetric {
    TieredIdentityMetric {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        tenant_id: "".to_string(),
        reviewer: "".to_string(),
        segment: "".to_string(),
        country: "".to_string(),
        owner_id: "".to_string(),
        source: "".to_string(),
        principal: "".to_string(),
        ratio: 0.0,
        status: TieredIdentityMetricStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_identity_metric("code-299", "name-409");
        assert_eq!(item.status, TieredIdentityMetricStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_tiered_identity_metric("code-299", "name-409");
        if item.move_to(TieredIdentityMetricStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredIdentityMetricStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_identity_metric("code-299", "name-409");
        item.status = TieredIdentityMetricStatus::Stopped;
        assert!(
            item.move_to(TieredIdentityMetricStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
