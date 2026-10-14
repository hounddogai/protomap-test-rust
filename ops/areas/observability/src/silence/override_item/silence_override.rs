use super::*;

use acme_kit::text;

/// Where a silence override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SilenceOverrideStatus {
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

impl std::fmt::Display for SilenceOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One silence override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceOverride {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub description: String,
    pub country: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub weight: f64,
    pub threshold: f64,
    pub tenant_id: String,
    pub status: SilenceOverrideStatus,
}

impl SilenceOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceOverrideStatus) -> bool {
        match self.status {
            SilenceOverrideStatus::Planned => {
                match next {
                    SilenceOverrideStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceOverrideStatus::Scheduled => {
                return next == SilenceOverrideStatus::InProgress;
            }
            SilenceOverrideStatus::InProgress => {
                return next == SilenceOverrideStatus::Completed || next == SilenceOverrideStatus::Cancelled;
            }
            SilenceOverrideStatus::Completed => {
                return next == SilenceOverrideStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "silence override", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_volume(&self) -> f64 {
        ((self.threshold - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceOverride {
        SilenceOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            description: self.description.clone(),
            country: self.country.clone(),
            tags: self.tags.clone(),
            pinned: self.pinned,
            weight: self.weight,
            threshold: self.threshold,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a silence override in status planned.
pub fn new_silence_override(code: &str, label: &str) -> SilenceOverride {
    SilenceOverride {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        description: "".to_string(),
        country: "".to_string(),
        tags: Vec::new(),
        pinned: false,
        weight: 0.0,
        threshold: 0.0,
        tenant_id: "".to_string(),
        status: SilenceOverrideStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_silence_override("code-999", "label-555");
        assert_eq!(item.status, SilenceOverrideStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_silence_override("code-999", "label-555");
        if item.move_to(SilenceOverrideStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceOverrideStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_silence_override("code-999", "label-555");
        item.status = SilenceOverrideStatus::Cancelled;
        assert!(
            item.move_to(SilenceOverrideStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
