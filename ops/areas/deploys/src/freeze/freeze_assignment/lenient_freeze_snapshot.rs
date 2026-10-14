use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient freeze snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientFreezeSnapshotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientFreezeSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientFreezeSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient freeze snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeSnapshot {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub description: String,
    pub checksum: String,
    pub tenant_id: String,
    pub country: String,
    pub margin: f64,
    pub scope: String,
    pub grace_days: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: LenientFreezeSnapshotStatus,
}

impl LenientFreezeSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientFreezeSnapshotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientFreezeSnapshotStatus) -> bool {
        match self.status {
            LenientFreezeSnapshotStatus::Requested => {
                match next {
                    LenientFreezeSnapshotStatus::Granted | LenientFreezeSnapshotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientFreezeSnapshotStatus::Granted => {
                match next {
                    LenientFreezeSnapshotStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientFreezeSnapshotStatus::Revoked => {
                match next {
                    LenientFreezeSnapshotStatus::Expired => {
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

    pub fn move_to(&mut self, next: LenientFreezeSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient freeze snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.margin + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientFreezeSnapshot {
        LenientFreezeSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            description: self.description.clone(),
            checksum: self.checksum.clone(),
            tenant_id: self.tenant_id.clone(),
            country: self.country.clone(),
            margin: self.margin,
            scope: self.scope.clone(),
            grace_days: self.grace_days,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a lenient freeze snapshot in status requested.
pub fn new_lenient_freeze_snapshot(code: &str, title: &str) -> LenientFreezeSnapshot {
    LenientFreezeSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        description: "".to_string(),
        checksum: "".to_string(),
        tenant_id: "".to_string(),
        country: "".to_string(),
        margin: 0.0,
        scope: "".to_string(),
        grace_days: 0,
        attributes: BTreeMap::new(),
        status: LenientFreezeSnapshotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_freeze_snapshot("code-28", "title-218");
        assert_eq!(item.status, LenientFreezeSnapshotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_lenient_freeze_snapshot("code-28", "title-218");
        if item.move_to(LenientFreezeSnapshotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientFreezeSnapshotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_freeze_snapshot("code-28", "title-218");
        item.status = LenientFreezeSnapshotStatus::Expired;
        assert!(
            item.move_to(LenientFreezeSnapshotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
