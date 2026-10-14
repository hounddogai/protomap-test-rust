use super::*;

use acme_kit::text;

/// Where a delegation mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DelegationMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DelegationMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a delegation mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMapping {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub starts_at: i64,
    pub retries: i64,
    pub source: String,
    pub priority: i64,
    pub expires_at: i64,
    pub status: DelegationMappingStatus,
}

impl DelegationMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationMappingStatus) -> bool {
        match self.status {
            DelegationMappingStatus::Proposed => {
                return next == DelegationMappingStatus::Withdrawn;
            }
            DelegationMappingStatus::Approved => {
                return next == DelegationMappingStatus::Withdrawn;
            }
            DelegationMappingStatus::Rejected => {
                return next == DelegationMappingStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.retries as f64 - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationMapping {
        DelegationMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            starts_at: self.starts_at,
            retries: self.retries,
            source: self.source.clone(),
            priority: self.priority,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a delegation mapping in status proposed.
pub fn new_delegation_mapping(code: &str, title: &str) -> DelegationMapping {
    DelegationMapping {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        starts_at: 0,
        retries: 0,
        source: "".to_string(),
        priority: 0,
        expires_at: 0,
        status: DelegationMappingStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_delegation_mapping("code-163", "title-847");
        assert_eq!(item.status, DelegationMappingStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_delegation_mapping("code-163", "title-847");
        if item.move_to(DelegationMappingStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationMappingStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_delegation_mapping("code-163", "title-847");
        item.status = DelegationMappingStatus::Withdrawn;
        assert!(
            item.move_to(DelegationMappingStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
