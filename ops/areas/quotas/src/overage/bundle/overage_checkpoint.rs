use super::*;

use acme_kit::text;

/// Where a overage checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl OverageCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for OverageCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage checkpoint as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageCheckpoint {
    pub code: String,
    pub label: String,
    pub name: String,
    pub retries: i64,
    pub approved: bool,
    pub minimum: f64,
    pub duration_minutes: i64,
    pub owner_id: String,
    pub audience: String,
    pub locale: String,
    pub status: OverageCheckpointStatus,
}

impl OverageCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageCheckpointStatus) -> bool {
        match self.status {
            OverageCheckpointStatus::Requested => {
                return next == OverageCheckpointStatus::Revoked;
            }
            OverageCheckpointStatus::Granted => {
                return next == OverageCheckpointStatus::Revoked || next == OverageCheckpointStatus::Expired;
            }
            OverageCheckpointStatus::Revoked => {
                match next {
                    OverageCheckpointStatus::Expired => {
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

    pub fn move_to(&mut self, next: OverageCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "overage checkpoint", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.minimum + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageCheckpoint {
        OverageCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            retries: self.retries,
            approved: self.approved,
            minimum: self.minimum,
            duration_minutes: self.duration_minutes,
            owner_id: self.owner_id.clone(),
            audience: self.audience.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a overage checkpoint in status requested.
pub fn new_overage_checkpoint(code: &str, label: &str) -> OverageCheckpoint {
    OverageCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        retries: 0,
        approved: false,
        minimum: 0.0,
        duration_minutes: 0,
        owner_id: "".to_string(),
        audience: "".to_string(),
        locale: "".to_string(),
        status: OverageCheckpointStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_overage_checkpoint("code-531", "label-913");
        assert_eq!(item.status, OverageCheckpointStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_overage_checkpoint("code-531", "label-913");
        if item.move_to(OverageCheckpointStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageCheckpointStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_overage_checkpoint("code-531", "label-913");
        item.status = OverageCheckpointStatus::Expired;
        assert!(
            item.move_to(OverageCheckpointStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
