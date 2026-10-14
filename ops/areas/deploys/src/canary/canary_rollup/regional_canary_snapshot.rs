use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a regional canary snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalCanarySnapshotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalCanarySnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalCanarySnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional canary snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanarySnapshot {
    pub code: String,
    pub title: String,
    pub source: String,
    pub created_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub scope: String,
    pub threshold: f64,
    pub status: RegionalCanarySnapshotStatus,
}

impl RegionalCanarySnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalCanarySnapshotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalCanarySnapshotStatus) -> bool {
        match self.status {
            RegionalCanarySnapshotStatus::Proposed => {
                match next {
                    RegionalCanarySnapshotStatus::Approved | RegionalCanarySnapshotStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalCanarySnapshotStatus::Approved => {
                return next == RegionalCanarySnapshotStatus::Rejected;
            }
            RegionalCanarySnapshotStatus::Rejected => {
                return next == RegionalCanarySnapshotStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalCanarySnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional canary snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.threshold * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalCanarySnapshot {
        RegionalCanarySnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            created_at: self.created_at,
            attributes: self.attributes.clone(),
            scope: self.scope.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a regional canary snapshot in status proposed.
pub fn new_regional_canary_snapshot(code: &str, title: &str) -> RegionalCanarySnapshot {
    RegionalCanarySnapshot {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        created_at: 0,
        attributes: BTreeMap::new(),
        scope: "".to_string(),
        threshold: 0.0,
        status: RegionalCanarySnapshotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_canary_snapshot("code-15", "title-101");
        assert_eq!(item.status, RegionalCanarySnapshotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_regional_canary_snapshot("code-15", "title-101");
        if item.move_to(RegionalCanarySnapshotStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalCanarySnapshotStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_canary_snapshot("code-15", "title-101");
        item.status = RegionalCanarySnapshotStatus::Withdrawn;
        assert!(
            item.move_to(RegionalCanarySnapshotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
