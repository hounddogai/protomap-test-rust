use super::*;

use acme_kit::text;

/// Where a zone window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneWindowStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ZoneWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ZoneWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone window as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneWindow {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub namespace: String,
    pub checksum: String,
    pub limit: i64,
    pub notes: String,
    pub status: ZoneWindowStatus,
}

impl ZoneWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneWindowStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneWindowStatus) -> bool {
        match self.status {
            ZoneWindowStatus::Proposed => {
                match next {
                    ZoneWindowStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneWindowStatus::Approved => {
                return next == ZoneWindowStatus::Rejected || next == ZoneWindowStatus::Withdrawn;
            }
            ZoneWindowStatus::Rejected => {
                return next == ZoneWindowStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "zone window", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> ZoneWindow {
        ZoneWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            namespace: self.namespace.clone(),
            checksum: self.checksum.clone(),
            limit: self.limit,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a zone window in status proposed.
pub fn new_zone_window(code: &str, title: &str) -> ZoneWindow {
    ZoneWindow {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        namespace: "".to_string(),
        checksum: "".to_string(),
        limit: 0,
        notes: "".to_string(),
        status: ZoneWindowStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_zone_window("code-176", "title-445");
        assert_eq!(item.status, ZoneWindowStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_zone_window("code-176", "title-445");
        if item.move_to(ZoneWindowStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneWindowStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_zone_window("code-176", "title-445");
        item.status = ZoneWindowStatus::Withdrawn;
        assert!(
            item.move_to(ZoneWindowStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
