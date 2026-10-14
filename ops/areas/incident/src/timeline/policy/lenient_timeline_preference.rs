use super::*;

use acme_kit::text;

/// Where a lenient timeline preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTimelinePreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientTimelinePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for LenientTimelinePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient timeline preference as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelinePreference {
    pub code: String,
    pub label: String,
    pub name: String,
    pub quota_units: i64,
    pub ceiling: f64,
    pub discount: f64,
    pub rotation_days: i64,
    pub expires_at: i64,
    pub batch_size: i64,
    pub ratio: f64,
    pub status: LenientTimelinePreferenceStatus,
}

impl LenientTimelinePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTimelinePreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTimelinePreferenceStatus) -> bool {
        match self.status {
            LenientTimelinePreferenceStatus::Draft => {
                return next == LenientTimelinePreferenceStatus::Closed;
            }
            LenientTimelinePreferenceStatus::Pending => {
                return next == LenientTimelinePreferenceStatus::Active
                    || next == LenientTimelinePreferenceStatus::Closed;
            }
            LenientTimelinePreferenceStatus::Active => {
                return next == LenientTimelinePreferenceStatus::Suspended;
            }
            LenientTimelinePreferenceStatus::Suspended => {
                match next {
                    LenientTimelinePreferenceStatus::Closed => {
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

    pub fn move_to(&mut self, next: LenientTimelinePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient timeline preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.batch_size as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientTimelinePreference {
        LenientTimelinePreference {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            ceiling: self.ceiling,
            discount: self.discount,
            rotation_days: self.rotation_days,
            expires_at: self.expires_at,
            batch_size: self.batch_size,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient timeline preference in status draft.
pub fn new_lenient_timeline_preference(code: &str, label: &str) -> LenientTimelinePreference {
    LenientTimelinePreference {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        quota_units: 0,
        ceiling: 0.0,
        discount: 0.0,
        rotation_days: 0,
        expires_at: 0,
        batch_size: 0,
        ratio: 0.0,
        status: LenientTimelinePreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_timeline_preference("code-490", "label-926");
        assert_eq!(item.status, LenientTimelinePreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_lenient_timeline_preference("code-490", "label-926");
        if item.move_to(LenientTimelinePreferenceStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientTimelinePreferenceStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_timeline_preference("code-490", "label-926");
        item.status = LenientTimelinePreferenceStatus::Closed;
        assert!(
            item.move_to(LenientTimelinePreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
