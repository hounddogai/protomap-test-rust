use super::*;

use acme_kit::text;

/// Where a adaptive plan snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptivePlanSnapshotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptivePlanSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptivePlanSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive plan snapshot as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanSnapshot {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub cipher: String,
    pub minimum: f64,
    pub namespace: String,
    pub score: f64,
    pub status: AdaptivePlanSnapshotStatus,
}

impl AdaptivePlanSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptivePlanSnapshotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptivePlanSnapshotStatus) -> bool {
        match self.status {
            AdaptivePlanSnapshotStatus::Requested => {
                match next {
                    AdaptivePlanSnapshotStatus::Revoked | AdaptivePlanSnapshotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptivePlanSnapshotStatus::Granted => {
                return next == AdaptivePlanSnapshotStatus::Revoked || next == AdaptivePlanSnapshotStatus::Expired;
            }
            AdaptivePlanSnapshotStatus::Revoked => {
                match next {
                    AdaptivePlanSnapshotStatus::Expired => {
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

    pub fn move_to(&mut self, next: AdaptivePlanSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive plan snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive plan snapshot", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.score - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptivePlanSnapshot {
        AdaptivePlanSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            cipher: self.cipher.clone(),
            minimum: self.minimum,
            namespace: self.namespace.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a adaptive plan snapshot in status requested.
pub fn new_adaptive_plan_snapshot(code: &str, title: &str) -> AdaptivePlanSnapshot {
    AdaptivePlanSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        cipher: "".to_string(),
        minimum: 0.0,
        namespace: "".to_string(),
        score: 0.0,
        status: AdaptivePlanSnapshotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_adaptive_plan_snapshot("code-480", "title-488");
        assert_eq!(item.status, AdaptivePlanSnapshotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_adaptive_plan_snapshot("code-480", "title-488");
        if item.move_to(AdaptivePlanSnapshotStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptivePlanSnapshotStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_adaptive_plan_snapshot("code-480", "title-488");
        item.status = AdaptivePlanSnapshotStatus::Expired;
        assert!(
            item.move_to(AdaptivePlanSnapshotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
