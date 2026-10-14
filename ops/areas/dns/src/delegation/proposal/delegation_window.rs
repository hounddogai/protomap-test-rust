use super::*;

use acme_kit::text;

/// Where a delegation window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationWindowStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DelegationWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DelegationWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationWindow {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub summary: String,
    pub cipher: String,
    pub owner_id: String,
    pub version: i64,
    pub checksum: String,
    pub ttl_seconds: i64,
    pub status: DelegationWindowStatus,
}

impl DelegationWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationWindowStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationWindowStatus) -> bool {
        match self.status {
            DelegationWindowStatus::Proposed => {
                return next == DelegationWindowStatus::Withdrawn;
            }
            DelegationWindowStatus::Approved => {
                match next {
                    DelegationWindowStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationWindowStatus::Rejected => {
                match next {
                    DelegationWindowStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DelegationWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationWindow {
        DelegationWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            summary: self.summary.clone(),
            cipher: self.cipher.clone(),
            owner_id: self.owner_id.clone(),
            version: self.version,
            checksum: self.checksum.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a delegation window in status proposed.
pub fn new_delegation_window(code: &str, name: &str) -> DelegationWindow {
    DelegationWindow {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        summary: "".to_string(),
        cipher: "".to_string(),
        owner_id: "".to_string(),
        version: 0,
        checksum: "".to_string(),
        ttl_seconds: 0,
        status: DelegationWindowStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_delegation_window("code-766", "name-103");
        assert_eq!(item.status, DelegationWindowStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_delegation_window("code-766", "name-103");
        if item.move_to(DelegationWindowStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationWindowStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_delegation_window("code-766", "name-103");
        item.status = DelegationWindowStatus::Withdrawn;
        assert!(
            item.move_to(DelegationWindowStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
