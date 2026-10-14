use super::*;

use acme_kit::text;

/// Where a egress estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressEstimateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EgressEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for EgressEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a egress estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressEstimate {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub revision: i64,
    pub quota_units: i64,
    pub active: bool,
    pub updated_at: i64,
    pub description: String,
    pub status: EgressEstimateStatus,
}

impl EgressEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressEstimateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressEstimateStatus) -> bool {
        match self.status {
            EgressEstimateStatus::Planned => {
                return next == EgressEstimateStatus::InProgress || next == EgressEstimateStatus::Cancelled;
            }
            EgressEstimateStatus::Scheduled => {
                match next {
                    EgressEstimateStatus::Completed | EgressEstimateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressEstimateStatus::InProgress => {
                return next == EgressEstimateStatus::Completed || next == EgressEstimateStatus::Cancelled;
            }
            EgressEstimateStatus::Completed => {
                match next {
                    EgressEstimateStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: EgressEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.port as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressEstimate {
        EgressEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            revision: self.revision,
            quota_units: self.quota_units,
            active: self.active,
            updated_at: self.updated_at,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a egress estimate in status planned.
pub fn new_egress_estimate(code: &str, name: &str) -> EgressEstimate {
    EgressEstimate {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        revision: 0,
        quota_units: 0,
        active: false,
        updated_at: 0,
        description: "".to_string(),
        status: EgressEstimateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_egress_estimate("code-865", "name-873");
        assert_eq!(item.status, EgressEstimateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_egress_estimate("code-865", "name-873");
        if item.move_to(EgressEstimateStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressEstimateStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_egress_estimate("code-865", "name-873");
        item.status = EgressEstimateStatus::Cancelled;
        assert!(
            item.move_to(EgressEstimateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
