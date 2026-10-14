use super::*;

use acme_kit::text;

/// Where a pending federation override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingFederationOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingFederationOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PendingFederationOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending federation override as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationOverride {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub quota_units: i64,
    pub reviewer: String,
    pub quantity: i64,
    pub external_ref: String,
    pub grace_days: i64,
    pub port: i64,
    pub status: PendingFederationOverrideStatus,
}

impl PendingFederationOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingFederationOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingFederationOverrideStatus) -> bool {
        match self.status {
            PendingFederationOverrideStatus::New => {
                return next == PendingFederationOverrideStatus::Running;
            }
            PendingFederationOverrideStatus::Queued => {
                match next {
                    PendingFederationOverrideStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingFederationOverrideStatus::Running => {
                return next == PendingFederationOverrideStatus::Done;
            }
            PendingFederationOverrideStatus::Done => {
                return next == PendingFederationOverrideStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingFederationOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending federation override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending federation override", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.quantity as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingFederationOverride {
        PendingFederationOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            quota_units: self.quota_units,
            reviewer: self.reviewer.clone(),
            quantity: self.quantity,
            external_ref: self.external_ref.clone(),
            grace_days: self.grace_days,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending federation override in status new.
pub fn new_pending_federation_override(code: &str, label: &str) -> PendingFederationOverride {
    PendingFederationOverride {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        quota_units: 0,
        reviewer: "".to_string(),
        quantity: 0,
        external_ref: "".to_string(),
        grace_days: 0,
        port: 0,
        status: PendingFederationOverrideStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_federation_override("code-629", "label-495");
        assert_eq!(item.status, PendingFederationOverrideStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_pending_federation_override("code-629", "label-495");
        if item.move_to(PendingFederationOverrideStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingFederationOverrideStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_federation_override("code-629", "label-495");
        item.status = PendingFederationOverrideStatus::Failed;
        assert!(
            item.move_to(PendingFederationOverrideStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
