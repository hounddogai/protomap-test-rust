use super::*;

use acme_kit::text;

/// Where a weighted zone checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedZoneCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl WeightedZoneCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for WeightedZoneCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted zone checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneCheckpoint {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub approved: bool,
    pub checksum: String,
    pub audience: String,
    pub tenant_id: String,
    pub version: i64,
    pub status: WeightedZoneCheckpointStatus,
}

impl WeightedZoneCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedZoneCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedZoneCheckpointStatus) -> bool {
        match self.status {
            WeightedZoneCheckpointStatus::Requested => {
                return next == WeightedZoneCheckpointStatus::Revoked || next == WeightedZoneCheckpointStatus::Expired;
            }
            WeightedZoneCheckpointStatus::Granted => {
                match next {
                    WeightedZoneCheckpointStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedZoneCheckpointStatus::Revoked => {
                return next == WeightedZoneCheckpointStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedZoneCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted zone checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.revision as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedZoneCheckpoint {
        WeightedZoneCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            approved: self.approved,
            checksum: self.checksum.clone(),
            audience: self.audience.clone(),
            tenant_id: self.tenant_id.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted zone checkpoint in status requested.
pub fn new_weighted_zone_checkpoint(code: &str, label: &str) -> WeightedZoneCheckpoint {
    WeightedZoneCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        approved: false,
        checksum: "".to_string(),
        audience: "".to_string(),
        tenant_id: "".to_string(),
        version: 0,
        status: WeightedZoneCheckpointStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_weighted_zone_checkpoint("code-102", "label-444");
        assert_eq!(item.status, WeightedZoneCheckpointStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_weighted_zone_checkpoint("code-102", "label-444");
        if item.move_to(WeightedZoneCheckpointStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedZoneCheckpointStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_weighted_zone_checkpoint("code-102", "label-444");
        item.status = WeightedZoneCheckpointStatus::Expired;
        assert!(
            item.move_to(WeightedZoneCheckpointStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
