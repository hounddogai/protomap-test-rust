use super::*;

use acme_kit::text;

/// Where a seasonal freeze snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalFreezeSnapshotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalFreezeSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalFreezeSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal freeze snapshot as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeSnapshot {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub maximum: f64,
    pub active: bool,
    pub port: i64,
    pub name: String,
    pub archived: bool,
    pub weight: f64,
    pub external_ref: String,
    pub locale: String,
    pub status: SeasonalFreezeSnapshotStatus,
}

impl SeasonalFreezeSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalFreezeSnapshotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalFreezeSnapshotStatus) -> bool {
        match self.status {
            SeasonalFreezeSnapshotStatus::Requested => {
                match next {
                    SeasonalFreezeSnapshotStatus::Revoked | SeasonalFreezeSnapshotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFreezeSnapshotStatus::Granted => {
                match next {
                    SeasonalFreezeSnapshotStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFreezeSnapshotStatus::Revoked => {
                match next {
                    SeasonalFreezeSnapshotStatus::Expired => {
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

    pub fn move_to(&mut self, next: SeasonalFreezeSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal freeze snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        (self.maximum * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalFreezeSnapshot {
        SeasonalFreezeSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            maximum: self.maximum,
            active: self.active,
            port: self.port,
            name: self.name.clone(),
            archived: self.archived,
            weight: self.weight,
            external_ref: self.external_ref.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a seasonal freeze snapshot in status requested.
pub fn new_seasonal_freeze_snapshot(code: &str, label: &str) -> SeasonalFreezeSnapshot {
    SeasonalFreezeSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        maximum: 0.0,
        active: false,
        port: 0,
        name: "".to_string(),
        archived: false,
        weight: 0.0,
        external_ref: "".to_string(),
        locale: "".to_string(),
        status: SeasonalFreezeSnapshotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_freeze_snapshot("code-747", "label-455");
        assert_eq!(item.status, SeasonalFreezeSnapshotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_seasonal_freeze_snapshot("code-747", "label-455");
        if item.move_to(SeasonalFreezeSnapshotStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalFreezeSnapshotStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_freeze_snapshot("code-747", "label-455");
        item.status = SeasonalFreezeSnapshotStatus::Expired;
        assert!(
            item.move_to(SeasonalFreezeSnapshotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
