use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a strict evidence entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictEvidenceEntryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictEvidenceEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for StrictEvidenceEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict evidence entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictEvidenceEntry {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub capacity: i64,
    pub locale: String,
    pub window_days: i64,
    pub attributes: BTreeMap<String, String>,
    pub unit: String,
    pub checksum: String,
    pub created_at: i64,
    pub issuer: String,
    pub status: StrictEvidenceEntryStatus,
}

impl StrictEvidenceEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictEvidenceEntryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictEvidenceEntryStatus) -> bool {
        match self.status {
            StrictEvidenceEntryStatus::Open => {
                match next {
                    StrictEvidenceEntryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictEvidenceEntryStatus::InReview => {
                return next == StrictEvidenceEntryStatus::Archived;
            }
            StrictEvidenceEntryStatus::Resolved => {
                match next {
                    StrictEvidenceEntryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictEvidenceEntryStatus::Reopened => {
                return next == StrictEvidenceEntryStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictEvidenceEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict evidence entry {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "strict evidence entry", self.code)
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

    pub fn projected_value(&self) -> f64 {
        ((self.window_days as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictEvidenceEntry {
        StrictEvidenceEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            capacity: self.capacity,
            locale: self.locale.clone(),
            window_days: self.window_days,
            attributes: self.attributes.clone(),
            unit: self.unit.clone(),
            checksum: self.checksum.clone(),
            created_at: self.created_at,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a strict evidence entry in status open.
pub fn new_strict_evidence_entry(code: &str, label: &str) -> StrictEvidenceEntry {
    StrictEvidenceEntry {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        capacity: 0,
        locale: "".to_string(),
        window_days: 0,
        attributes: BTreeMap::new(),
        unit: "".to_string(),
        checksum: "".to_string(),
        created_at: 0,
        issuer: "".to_string(),
        status: StrictEvidenceEntryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_evidence_entry("code-444", "label-992");
        assert_eq!(item.status, StrictEvidenceEntryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_strict_evidence_entry("code-444", "label-992");
        if item.move_to(StrictEvidenceEntryStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictEvidenceEntryStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_evidence_entry("code-444", "label-992");
        item.status = StrictEvidenceEntryStatus::Archived;
        assert!(
            item.move_to(StrictEvidenceEntryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
