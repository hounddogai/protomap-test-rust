use super::*;

use acme_kit::text;

/// Where a manual peering snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualPeeringSnapshotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualPeeringSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualPeeringSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual peering snapshot as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringSnapshot {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub grace_days: i64,
    pub pinned: bool,
    pub channel: String,
    pub contact_email: String,
    pub status: ManualPeeringSnapshotStatus,
}

impl ManualPeeringSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualPeeringSnapshotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualPeeringSnapshotStatus) -> bool {
        match self.status {
            ManualPeeringSnapshotStatus::Created => {
                return next == ManualPeeringSnapshotStatus::Published;
            }
            ManualPeeringSnapshotStatus::Validated => {
                return next == ManualPeeringSnapshotStatus::Published || next == ManualPeeringSnapshotStatus::Retired;
            }
            ManualPeeringSnapshotStatus::Published => {
                return next == ManualPeeringSnapshotStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualPeeringSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual peering snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.grace_days as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualPeeringSnapshot {
        ManualPeeringSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            grace_days: self.grace_days,
            pinned: self.pinned,
            channel: self.channel.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a manual peering snapshot in status created.
pub fn new_manual_peering_snapshot(code: &str, label: &str) -> ManualPeeringSnapshot {
    ManualPeeringSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        grace_days: 0,
        pinned: false,
        channel: "".to_string(),
        contact_email: "".to_string(),
        status: ManualPeeringSnapshotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_peering_snapshot("code-153", "label-914");
        assert_eq!(item.status, ManualPeeringSnapshotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_manual_peering_snapshot("code-153", "label-914");
        if item.move_to(ManualPeeringSnapshotStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualPeeringSnapshotStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_peering_snapshot("code-153", "label-914");
        item.status = ManualPeeringSnapshotStatus::Retired;
        assert!(
            item.move_to(ManualPeeringSnapshotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
