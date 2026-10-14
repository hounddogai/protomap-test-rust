use super::*;

use acme_kit::text;

/// Where a delegation metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationMetricStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DelegationMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DelegationMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation metric as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMetric {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub region: String,
    pub fingerprint: String,
    pub port: i64,
    pub priority: i64,
    pub cipher: String,
    pub rate: f64,
    pub visible: bool,
    pub channel: String,
    pub status: DelegationMetricStatus,
}

impl DelegationMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationMetricStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationMetricStatus) -> bool {
        match self.status {
            DelegationMetricStatus::Proposed => {
                return next == DelegationMetricStatus::Rejected;
            }
            DelegationMetricStatus::Approved => {
                return next == DelegationMetricStatus::Withdrawn;
            }
            DelegationMetricStatus::Rejected => {
                return next == DelegationMetricStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.port as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationMetric {
        DelegationMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            region: self.region.clone(),
            fingerprint: self.fingerprint.clone(),
            port: self.port,
            priority: self.priority,
            cipher: self.cipher.clone(),
            rate: self.rate,
            visible: self.visible,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a delegation metric in status proposed.
pub fn new_delegation_metric(code: &str, name: &str) -> DelegationMetric {
    DelegationMetric {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        region: "".to_string(),
        fingerprint: "".to_string(),
        port: 0,
        priority: 0,
        cipher: "".to_string(),
        rate: 0.0,
        visible: false,
        channel: "".to_string(),
        status: DelegationMetricStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_delegation_metric("code-178", "name-1");
        assert_eq!(item.status, DelegationMetricStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_delegation_metric("code-178", "name-1");
        if item.move_to(DelegationMetricStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationMetricStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_delegation_metric("code-178", "name-1");
        item.status = DelegationMetricStatus::Withdrawn;
        assert!(
            item.move_to(DelegationMetricStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
