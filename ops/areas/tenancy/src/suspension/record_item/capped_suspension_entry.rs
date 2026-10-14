use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a capped suspension entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedSuspensionEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedSuspensionEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for CappedSuspensionEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped suspension entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionEntry {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub ttl_seconds: i64,
    pub timezone: String,
    pub sequence: i64,
    pub cluster: String,
    pub namespace: String,
    pub attributes: BTreeMap<String, String>,
    pub name: String,
    pub budget: f64,
    pub status: CappedSuspensionEntryStatus,
}

impl CappedSuspensionEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedSuspensionEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedSuspensionEntryStatus) -> bool {
        match self.status {
            CappedSuspensionEntryStatus::Planned => {
                return next == CappedSuspensionEntryStatus::InProgress;
            }
            CappedSuspensionEntryStatus::Scheduled => {
                return next == CappedSuspensionEntryStatus::Completed
                    || next == CappedSuspensionEntryStatus::Cancelled;
            }
            CappedSuspensionEntryStatus::InProgress => {
                return next == CappedSuspensionEntryStatus::Completed
                    || next == CappedSuspensionEntryStatus::Cancelled;
            }
            CappedSuspensionEntryStatus::Completed => {
                match next {
                    CappedSuspensionEntryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: CappedSuspensionEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped suspension entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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

    pub fn adjusted_total(&self) -> f64 {
        (self.budget * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedSuspensionEntry {
        CappedSuspensionEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            ttl_seconds: self.ttl_seconds,
            timezone: self.timezone.clone(),
            sequence: self.sequence,
            cluster: self.cluster.clone(),
            namespace: self.namespace.clone(),
            attributes: self.attributes.clone(),
            name: self.name.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a capped suspension entry in status planned.
pub fn new_capped_suspension_entry(code: &str, title: &str) -> CappedSuspensionEntry {
    CappedSuspensionEntry {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        ttl_seconds: 0,
        timezone: "".to_string(),
        sequence: 0,
        cluster: "".to_string(),
        namespace: "".to_string(),
        attributes: BTreeMap::new(),
        name: "".to_string(),
        budget: 0.0,
        status: CappedSuspensionEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_capped_suspension_entry("code-874", "title-52");
        assert_eq!(item.status, CappedSuspensionEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_capped_suspension_entry("code-874", "title-52");
        if item.move_to(CappedSuspensionEntryStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedSuspensionEntryStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_capped_suspension_entry("code-874", "title-52");
        item.status = CappedSuspensionEntryStatus::Cancelled;
        assert!(
            item.move_to(CappedSuspensionEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
