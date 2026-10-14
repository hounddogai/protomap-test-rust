use super::*;

use acme_kit::text;

/// Where a capped overage snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedOverageSnapshotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedOverageSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedOverageSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped overage snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageSnapshot {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub unit: String,
    pub ratio: f64,
    pub batch_size: i64,
    pub principal: String,
    pub status: CappedOverageSnapshotStatus,
}

impl CappedOverageSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedOverageSnapshotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedOverageSnapshotStatus) -> bool {
        match self.status {
            CappedOverageSnapshotStatus::Requested => {
                match next {
                    CappedOverageSnapshotStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedOverageSnapshotStatus::Granted => {
                return next == CappedOverageSnapshotStatus::Revoked || next == CappedOverageSnapshotStatus::Expired;
            }
            CappedOverageSnapshotStatus::Revoked => {
                match next {
                    CappedOverageSnapshotStatus::Expired => {
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

    pub fn move_to(&mut self, next: CappedOverageSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped overage snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ratio - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedOverageSnapshot {
        CappedOverageSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            unit: self.unit.clone(),
            ratio: self.ratio,
            batch_size: self.batch_size,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a capped overage snapshot in status requested.
pub fn new_capped_overage_snapshot(code: &str, label: &str) -> CappedOverageSnapshot {
    CappedOverageSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        unit: "".to_string(),
        ratio: 0.0,
        batch_size: 0,
        principal: "".to_string(),
        status: CappedOverageSnapshotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_overage_snapshot("code-772", "label-953");
        assert_eq!(item.status, CappedOverageSnapshotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_capped_overage_snapshot("code-772", "label-953");
        if item.move_to(CappedOverageSnapshotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedOverageSnapshotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_overage_snapshot("code-772", "label-953");
        item.status = CappedOverageSnapshotStatus::Expired;
        assert!(
            item.move_to(CappedOverageSnapshotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
