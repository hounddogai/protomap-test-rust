use super::*;

use acme_kit::text;

/// Where a lenient alert override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientAlertOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientAlertOverrideStatus {
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

impl std::fmt::Display for LenientAlertOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient alert override as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientAlertOverride {
    pub code: String,
    pub label: String,
    pub source: String,
    pub namespace: String,
    pub principal: String,
    pub window_days: i64,
    pub summary: String,
    pub owner_id: String,
    pub budget: f64,
    pub visible: bool,
    pub ttl_seconds: i64,
    pub status: LenientAlertOverrideStatus,
}

impl LenientAlertOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientAlertOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientAlertOverrideStatus) -> bool {
        match self.status {
            LenientAlertOverrideStatus::Planned => {
                match next {
                    LenientAlertOverrideStatus::Scheduled | LenientAlertOverrideStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientAlertOverrideStatus::Scheduled => {
                return next == LenientAlertOverrideStatus::InProgress;
            }
            LenientAlertOverrideStatus::InProgress => {
                match next {
                    LenientAlertOverrideStatus::Completed | LenientAlertOverrideStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientAlertOverrideStatus::Completed => {
                match next {
                    LenientAlertOverrideStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LenientAlertOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient alert override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient alert override", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.budget - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientAlertOverride {
        LenientAlertOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            namespace: self.namespace.clone(),
            principal: self.principal.clone(),
            window_days: self.window_days,
            summary: self.summary.clone(),
            owner_id: self.owner_id.clone(),
            budget: self.budget,
            visible: self.visible,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a lenient alert override in status planned.
pub fn new_lenient_alert_override(code: &str, label: &str) -> LenientAlertOverride {
    LenientAlertOverride {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        namespace: "".to_string(),
        principal: "".to_string(),
        window_days: 0,
        summary: "".to_string(),
        owner_id: "".to_string(),
        budget: 0.0,
        visible: false,
        ttl_seconds: 0,
        status: LenientAlertOverrideStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_alert_override("code-761", "label-275");
        assert_eq!(item.status, LenientAlertOverrideStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_lenient_alert_override("code-761", "label-275");
        if item.move_to(LenientAlertOverrideStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientAlertOverrideStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_alert_override("code-761", "label-275");
        item.status = LenientAlertOverrideStatus::Cancelled;
        assert!(
            item.move_to(LenientAlertOverrideStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
