use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a dns record preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordPreferenceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DnsRecordPreferenceStatus {
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

impl std::fmt::Display for DnsRecordPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dns record preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPreference {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub attributes: BTreeMap<String, String>,
    pub tenant_id: String,
    pub pinned: bool,
    pub floor_value: f64,
    pub weight: f64,
    pub issuer: String,
    pub status: DnsRecordPreferenceStatus,
}

impl DnsRecordPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordPreferenceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordPreferenceStatus) -> bool {
        match self.status {
            DnsRecordPreferenceStatus::Planned => {
                return next == DnsRecordPreferenceStatus::InProgress || next == DnsRecordPreferenceStatus::Cancelled;
            }
            DnsRecordPreferenceStatus::Scheduled => {
                match next {
                    DnsRecordPreferenceStatus::Completed | DnsRecordPreferenceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordPreferenceStatus::InProgress => {
                return next == DnsRecordPreferenceStatus::Completed;
            }
            DnsRecordPreferenceStatus::Completed => {
                return next == DnsRecordPreferenceStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
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

    pub fn projected_value(&self) -> f64 {
        ((self.priority as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordPreference {
        DnsRecordPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            attributes: self.attributes.clone(),
            tenant_id: self.tenant_id.clone(),
            pinned: self.pinned,
            floor_value: self.floor_value,
            weight: self.weight,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dns record preference in status planned.
pub fn new_dns_record_preference(code: &str, label: &str) -> DnsRecordPreference {
    DnsRecordPreference {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        attributes: BTreeMap::new(),
        tenant_id: "".to_string(),
        pinned: false,
        floor_value: 0.0,
        weight: 0.0,
        issuer: "".to_string(),
        status: DnsRecordPreferenceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_dns_record_preference("code-868", "label-39");
        assert_eq!(item.status, DnsRecordPreferenceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_dns_record_preference("code-868", "label-39");
        if item.move_to(DnsRecordPreferenceStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordPreferenceStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_dns_record_preference("code-868", "label-39");
        item.status = DnsRecordPreferenceStatus::Cancelled;
        assert!(
            item.move_to(DnsRecordPreferenceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
