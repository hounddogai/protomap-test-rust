use super::*;

use acme_kit::text;

/// Where a default zone snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultZoneSnapshotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultZoneSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultZoneSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default zone snapshot as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneSnapshot {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub visible: bool,
    pub margin: f64,
    pub summary: String,
    pub reviewer: String,
    pub ceiling: f64,
    pub status: DefaultZoneSnapshotStatus,
}

impl DefaultZoneSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultZoneSnapshotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultZoneSnapshotStatus) -> bool {
        match self.status {
            DefaultZoneSnapshotStatus::Requested => {
                match next {
                    DefaultZoneSnapshotStatus::Granted | DefaultZoneSnapshotStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultZoneSnapshotStatus::Granted => {
                match next {
                    DefaultZoneSnapshotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultZoneSnapshotStatus::Revoked => {
                return next == DefaultZoneSnapshotStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultZoneSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default zone snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.ceiling - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultZoneSnapshot {
        DefaultZoneSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            visible: self.visible,
            margin: self.margin,
            summary: self.summary.clone(),
            reviewer: self.reviewer.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default zone snapshot in status requested.
pub fn new_default_zone_snapshot(code: &str, title: &str) -> DefaultZoneSnapshot {
    DefaultZoneSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        visible: false,
        margin: 0.0,
        summary: "".to_string(),
        reviewer: "".to_string(),
        ceiling: 0.0,
        status: DefaultZoneSnapshotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_zone_snapshot("code-74", "title-948");
        assert_eq!(item.status, DefaultZoneSnapshotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_default_zone_snapshot("code-74", "title-948");
        if item.move_to(DefaultZoneSnapshotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultZoneSnapshotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_zone_snapshot("code-74", "title-948");
        item.status = DefaultZoneSnapshotStatus::Expired;
        assert!(
            item.move_to(DefaultZoneSnapshotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
