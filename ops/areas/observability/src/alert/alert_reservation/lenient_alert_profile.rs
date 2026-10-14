use super::*;

use acme_kit::text;

/// Where a lenient alert profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientAlertProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientAlertProfileStatus {
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

impl std::fmt::Display for LenientAlertProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient alert profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientAlertProfile {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub tags: Vec<String>,
    pub name: String,
    pub ttl_seconds: i64,
    pub label: String,
    pub duration_minutes: i64,
    pub key_id: String,
    pub status: LenientAlertProfileStatus,
}

impl LenientAlertProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientAlertProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientAlertProfileStatus) -> bool {
        match self.status {
            LenientAlertProfileStatus::Planned => {
                return next == LenientAlertProfileStatus::Completed;
            }
            LenientAlertProfileStatus::Scheduled => {
                match next {
                    LenientAlertProfileStatus::InProgress | LenientAlertProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientAlertProfileStatus::InProgress => {
                return next == LenientAlertProfileStatus::Completed;
            }
            LenientAlertProfileStatus::Completed => {
                return next == LenientAlertProfileStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientAlertProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient alert profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientAlertProfile {
        LenientAlertProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            tags: self.tags.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient alert profile in status planned.
pub fn new_lenient_alert_profile(code: &str, title: &str) -> LenientAlertProfile {
    LenientAlertProfile {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        tags: Vec::new(),
        name: "".to_string(),
        ttl_seconds: 0,
        label: "".to_string(),
        duration_minutes: 0,
        key_id: "".to_string(),
        status: LenientAlertProfileStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_alert_profile("code-432", "title-793");
        assert_eq!(item.status, LenientAlertProfileStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_lenient_alert_profile("code-432", "title-793");
        if item.move_to(LenientAlertProfileStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientAlertProfileStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_alert_profile("code-432", "title-793");
        item.status = LenientAlertProfileStatus::Cancelled;
        assert!(
            item.move_to(LenientAlertProfileStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
