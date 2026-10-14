use super::*;

use acme_kit::text;

/// Where a audit event override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AuditEventOverrideStatus {
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

impl std::fmt::Display for AuditEventOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event override as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventOverride {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub source: String,
    pub window_days: i64,
    pub unit: String,
    pub category: String,
    pub maximum: f64,
    pub duration_minutes: i64,
    pub audience: String,
    pub rotation_days: i64,
    pub status: AuditEventOverrideStatus,
}

impl AuditEventOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventOverrideStatus) -> bool {
        match self.status {
            AuditEventOverrideStatus::Planned => {
                match next {
                    AuditEventOverrideStatus::Scheduled | AuditEventOverrideStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventOverrideStatus::Scheduled => {
                return next == AuditEventOverrideStatus::Cancelled;
            }
            AuditEventOverrideStatus::InProgress => {
                return next == AuditEventOverrideStatus::Completed || next == AuditEventOverrideStatus::Cancelled;
            }
            AuditEventOverrideStatus::Completed => {
                match next {
                    AuditEventOverrideStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: AuditEventOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.updated_at as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventOverride {
        AuditEventOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            source: self.source.clone(),
            window_days: self.window_days,
            unit: self.unit.clone(),
            category: self.category.clone(),
            maximum: self.maximum,
            duration_minutes: self.duration_minutes,
            audience: self.audience.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a audit event override in status planned.
pub fn new_audit_event_override(code: &str, title: &str) -> AuditEventOverride {
    AuditEventOverride {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        source: "".to_string(),
        window_days: 0,
        unit: "".to_string(),
        category: "".to_string(),
        maximum: 0.0,
        duration_minutes: 0,
        audience: "".to_string(),
        rotation_days: 0,
        status: AuditEventOverrideStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_audit_event_override("code-486", "title-303");
        assert_eq!(item.status, AuditEventOverrideStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_audit_event_override("code-486", "title-303");
        if item.move_to(AuditEventOverrideStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventOverrideStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_audit_event_override("code-486", "title-303");
        item.status = AuditEventOverrideStatus::Cancelled;
        assert!(
            item.move_to(AuditEventOverrideStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
