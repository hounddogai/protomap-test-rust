use super::*;

use acme_kit::text;

/// Where a trust domain snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainSnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TrustDomainSnapshotStatus {
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

impl std::fmt::Display for TrustDomainSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain snapshot as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSnapshot {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub timezone: String,
    pub attempts: i64,
    pub minimum: f64,
    pub capacity: i64,
    pub margin: f64,
    pub window_days: i64,
    pub principal: String,
    pub pinned: bool,
    pub status: TrustDomainSnapshotStatus,
}

impl TrustDomainSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainSnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainSnapshotStatus) -> bool {
        match self.status {
            TrustDomainSnapshotStatus::Planned => {
                match next {
                    TrustDomainSnapshotStatus::Scheduled | TrustDomainSnapshotStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainSnapshotStatus::Scheduled => {
                match next {
                    TrustDomainSnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainSnapshotStatus::InProgress => {
                match next {
                    TrustDomainSnapshotStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainSnapshotStatus::Completed => {
                return next == TrustDomainSnapshotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.window_days as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainSnapshot {
        TrustDomainSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            timezone: self.timezone.clone(),
            attempts: self.attempts,
            minimum: self.minimum,
            capacity: self.capacity,
            margin: self.margin,
            window_days: self.window_days,
            principal: self.principal.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a trust domain snapshot in status planned.
pub fn new_trust_domain_snapshot(code: &str, label: &str) -> TrustDomainSnapshot {
    TrustDomainSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        timezone: "".to_string(),
        attempts: 0,
        minimum: 0.0,
        capacity: 0,
        margin: 0.0,
        window_days: 0,
        principal: "".to_string(),
        pinned: false,
        status: TrustDomainSnapshotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_trust_domain_snapshot("code-902", "label-86");
        assert_eq!(item.status, TrustDomainSnapshotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_trust_domain_snapshot("code-902", "label-86");
        if item.move_to(TrustDomainSnapshotStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainSnapshotStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_trust_domain_snapshot("code-902", "label-86");
        item.status = TrustDomainSnapshotStatus::Cancelled;
        assert!(
            item.move_to(TrustDomainSnapshotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
