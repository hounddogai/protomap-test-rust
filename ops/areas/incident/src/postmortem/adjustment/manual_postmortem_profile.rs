use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a manual postmortem profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualPostmortemProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualPostmortemProfileStatus {
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

impl std::fmt::Display for ManualPostmortemProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual postmortem profile as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPostmortemProfile {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub attributes: BTreeMap<String, String>,
    pub segment: String,
    pub quota_units: i64,
    pub status: ManualPostmortemProfileStatus,
}

impl ManualPostmortemProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualPostmortemProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualPostmortemProfileStatus) -> bool {
        match self.status {
            ManualPostmortemProfileStatus::New => {
                return next == ManualPostmortemProfileStatus::Queued;
            }
            ManualPostmortemProfileStatus::Queued => {
                return next == ManualPostmortemProfileStatus::Done;
            }
            ManualPostmortemProfileStatus::Running => {
                return next == ManualPostmortemProfileStatus::Done || next == ManualPostmortemProfileStatus::Failed;
            }
            ManualPostmortemProfileStatus::Done => {
                match next {
                    ManualPostmortemProfileStatus::Failed => {
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

    pub fn move_to(&mut self, next: ManualPostmortemProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual postmortem profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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
        self.quota_units as f64
    }

    pub fn copy(&self) -> ManualPostmortemProfile {
        ManualPostmortemProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            attributes: self.attributes.clone(),
            segment: self.segment.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a manual postmortem profile in status new.
pub fn new_manual_postmortem_profile(code: &str, label: &str) -> ManualPostmortemProfile {
    ManualPostmortemProfile {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        attributes: BTreeMap::new(),
        segment: "".to_string(),
        quota_units: 0,
        status: ManualPostmortemProfileStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_postmortem_profile("code-730", "label-228");
        assert_eq!(item.status, ManualPostmortemProfileStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_manual_postmortem_profile("code-730", "label-228");
        if item.move_to(ManualPostmortemProfileStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualPostmortemProfileStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_postmortem_profile("code-730", "label-228");
        item.status = ManualPostmortemProfileStatus::Failed;
        assert!(
            item.move_to(ManualPostmortemProfileStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
