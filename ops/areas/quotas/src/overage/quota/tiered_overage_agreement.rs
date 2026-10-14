use super::*;

use acme_kit::text;

/// Where a tiered overage agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredOverageAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredOverageAgreementStatus {
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

impl std::fmt::Display for TieredOverageAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered overage agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageAgreement {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub grace_days: i64,
    pub reviewer: String,
    pub external_ref: String,
    pub audience: String,
    pub cluster: String,
    pub port: i64,
    pub ends_at: i64,
    pub namespace: String,
    pub status: TieredOverageAgreementStatus,
}

impl TieredOverageAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredOverageAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredOverageAgreementStatus) -> bool {
        match self.status {
            TieredOverageAgreementStatus::Planned => {
                return next == TieredOverageAgreementStatus::Completed;
            }
            TieredOverageAgreementStatus::Scheduled => {
                match next {
                    TieredOverageAgreementStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredOverageAgreementStatus::InProgress => {
                return next == TieredOverageAgreementStatus::Completed
                    || next == TieredOverageAgreementStatus::Cancelled;
            }
            TieredOverageAgreementStatus::Completed => {
                return next == TieredOverageAgreementStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredOverageAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered overage agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.grace_days as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredOverageAgreement {
        TieredOverageAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            grace_days: self.grace_days,
            reviewer: self.reviewer.clone(),
            external_ref: self.external_ref.clone(),
            audience: self.audience.clone(),
            cluster: self.cluster.clone(),
            port: self.port,
            ends_at: self.ends_at,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a tiered overage agreement in status planned.
pub fn new_tiered_overage_agreement(code: &str, name: &str) -> TieredOverageAgreement {
    TieredOverageAgreement {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        grace_days: 0,
        reviewer: "".to_string(),
        external_ref: "".to_string(),
        audience: "".to_string(),
        cluster: "".to_string(),
        port: 0,
        ends_at: 0,
        namespace: "".to_string(),
        status: TieredOverageAgreementStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_overage_agreement("code-973", "name-412");
        assert_eq!(item.status, TieredOverageAgreementStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_tiered_overage_agreement("code-973", "name-412");
        if item.move_to(TieredOverageAgreementStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredOverageAgreementStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_overage_agreement("code-973", "name-412");
        item.status = TieredOverageAgreementStatus::Cancelled;
        assert!(
            item.move_to(TieredOverageAgreementStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
