use super::*;

use acme_kit::text;

/// Where a strict egress preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictEgressPreferenceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictEgressPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for StrictEgressPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict egress preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressPreference {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub country: String,
    pub segment: String,
    pub key_id: String,
    pub maximum: f64,
    pub owner_id: String,
    pub attempts: i64,
    pub revision: i64,
    pub tags: Vec<String>,
    pub status: StrictEgressPreferenceStatus,
}

impl StrictEgressPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictEgressPreferenceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictEgressPreferenceStatus) -> bool {
        match self.status {
            StrictEgressPreferenceStatus::New => {
                match next {
                    StrictEgressPreferenceStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictEgressPreferenceStatus::Queued => {
                match next {
                    StrictEgressPreferenceStatus::Done | StrictEgressPreferenceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictEgressPreferenceStatus::Running => {
                return next == StrictEgressPreferenceStatus::Done || next == StrictEgressPreferenceStatus::Failed;
            }
            StrictEgressPreferenceStatus::Done => {
                return next == StrictEgressPreferenceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictEgressPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict egress preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "strict egress preference", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_value(&self) -> f64 {
        ((self.rotation_days as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictEgressPreference {
        StrictEgressPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            country: self.country.clone(),
            segment: self.segment.clone(),
            key_id: self.key_id.clone(),
            maximum: self.maximum,
            owner_id: self.owner_id.clone(),
            attempts: self.attempts,
            revision: self.revision,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a strict egress preference in status new.
pub fn new_strict_egress_preference(code: &str, label: &str) -> StrictEgressPreference {
    StrictEgressPreference {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        country: "".to_string(),
        segment: "".to_string(),
        key_id: "".to_string(),
        maximum: 0.0,
        owner_id: "".to_string(),
        attempts: 0,
        revision: 0,
        tags: Vec::new(),
        status: StrictEgressPreferenceStatus::New,
    }
}
