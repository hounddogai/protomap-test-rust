use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a evidence plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidencePlanStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl EvidencePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for EvidencePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePlan {
    pub code: String,
    pub label: String,
    pub source: String,
    pub title: String,
    pub port: i64,
    pub phone: String,
    pub pinned: bool,
    pub channel: String,
    pub attributes: BTreeMap<String, String>,
    pub status: EvidencePlanStatus,
}

impl EvidencePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidencePlanStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidencePlanStatus) -> bool {
        match self.status {
            EvidencePlanStatus::Proposed => {
                match next {
                    EvidencePlanStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidencePlanStatus::Approved => {
                return next == EvidencePlanStatus::Rejected || next == EvidencePlanStatus::Withdrawn;
            }
            EvidencePlanStatus::Rejected => {
                return next == EvidencePlanStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidencePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "evidence plan", self.code)
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

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> EvidencePlan {
        EvidencePlan {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            title: self.title.clone(),
            port: self.port,
            phone: self.phone.clone(),
            pinned: self.pinned,
            channel: self.channel.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a evidence plan in status proposed.
pub fn new_evidence_plan(code: &str, label: &str) -> EvidencePlan {
    EvidencePlan {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        title: "".to_string(),
        port: 0,
        phone: "".to_string(),
        pinned: false,
        channel: "".to_string(),
        attributes: BTreeMap::new(),
        status: EvidencePlanStatus::Proposed,
    }
}
