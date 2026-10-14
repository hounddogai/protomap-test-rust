use super::*;

use acme_kit::text;

/// Where a retention mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RetentionMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RetentionMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention mapping as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMapping {
    pub code: String,
    pub name: String,
    pub title: String,
    pub region: String,
    pub scope: String,
    pub owner_id: String,
    pub source: String,
    pub sequence: i64,
    pub status: RetentionMappingStatus,
}

impl RetentionMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionMappingStatus) -> bool {
        match self.status {
            RetentionMappingStatus::Requested => {
                match next {
                    RetentionMappingStatus::Revoked | RetentionMappingStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionMappingStatus::Granted => {
                return next == RetentionMappingStatus::Revoked || next == RetentionMappingStatus::Expired;
            }
            RetentionMappingStatus::Revoked => {
                match next {
                    RetentionMappingStatus::Expired => {
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

    pub fn move_to(&mut self, next: RetentionMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> RetentionMapping {
        RetentionMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            scope: self.scope.clone(),
            owner_id: self.owner_id.clone(),
            source: self.source.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a retention mapping in status requested.
pub fn new_retention_mapping(code: &str, name: &str) -> RetentionMapping {
    RetentionMapping {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        region: "".to_string(),
        scope: "".to_string(),
        owner_id: "".to_string(),
        source: "".to_string(),
        sequence: 0,
        status: RetentionMappingStatus::Requested,
    }
}
