use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a freeze record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FreezeRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FreezeRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze record as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRecord {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub limit: i64,
    pub description: String,
    pub updated_at: i64,
    pub reviewer: String,
    pub issuer: String,
    pub attributes: BTreeMap<String, String>,
    pub floor_value: f64,
    pub status: FreezeRecordStatus,
}

impl FreezeRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeRecordStatus) -> bool {
        match self.status {
            FreezeRecordStatus::Proposed => {
                return next == FreezeRecordStatus::Rejected || next == FreezeRecordStatus::Withdrawn;
            }
            FreezeRecordStatus::Approved => {
                return next == FreezeRecordStatus::Rejected || next == FreezeRecordStatus::Withdrawn;
            }
            FreezeRecordStatus::Rejected => {
                match next {
                    FreezeRecordStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FreezeRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze record {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
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

    pub fn weighted_value(&self) -> f64 {
        ((self.limit as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeRecord {
        FreezeRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            limit: self.limit,
            description: self.description.clone(),
            updated_at: self.updated_at,
            reviewer: self.reviewer.clone(),
            issuer: self.issuer.clone(),
            attributes: self.attributes.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a freeze record in status proposed.
pub fn new_freeze_record(code: &str, label: &str) -> FreezeRecord {
    FreezeRecord {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        limit: 0,
        description: "".to_string(),
        updated_at: 0,
        reviewer: "".to_string(),
        issuer: "".to_string(),
        attributes: BTreeMap::new(),
        floor_value: 0.0,
        status: FreezeRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_freeze_record("code-849", "label-111");
        assert_eq!(item.status, FreezeRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_freeze_record("code-849", "label-111");
        if item.move_to(FreezeRecordStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeRecordStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_freeze_record("code-849", "label-111");
        item.status = FreezeRecordStatus::Withdrawn;
        assert!(
            item.move_to(FreezeRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
