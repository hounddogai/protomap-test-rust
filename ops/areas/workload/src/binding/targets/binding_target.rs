use super::*;

use acme_kit::text;

/// Where a binding target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingTargetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BindingTargetStatus {
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

impl std::fmt::Display for BindingTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingTarget {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub channel: String,
    pub source: String,
    pub visible: bool,
    pub threshold: f64,
    pub score: f64,
    pub created_at: i64,
    pub contact_email: String,
    pub status: BindingTargetStatus,
}

impl BindingTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingTargetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingTargetStatus) -> bool {
        match self.status {
            BindingTargetStatus::Planned => {
                return next == BindingTargetStatus::InProgress;
            }
            BindingTargetStatus::Scheduled => {
                return next == BindingTargetStatus::Completed || next == BindingTargetStatus::Cancelled;
            }
            BindingTargetStatus::InProgress => {
                return next == BindingTargetStatus::Completed;
            }
            BindingTargetStatus::Completed => {
                return next == BindingTargetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.score - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingTarget {
        BindingTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            channel: self.channel.clone(),
            source: self.source.clone(),
            visible: self.visible,
            threshold: self.threshold,
            score: self.score,
            created_at: self.created_at,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a binding target in status planned.
pub fn new_binding_target(code: &str, title: &str) -> BindingTarget {
    BindingTarget {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        channel: "".to_string(),
        source: "".to_string(),
        visible: false,
        threshold: 0.0,
        score: 0.0,
        created_at: 0,
        contact_email: "".to_string(),
        status: BindingTargetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_binding_target("code-318", "title-439");
        assert_eq!(item.status, BindingTargetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_binding_target("code-318", "title-439");
        if item.move_to(BindingTargetStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingTargetStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_binding_target("code-318", "title-439");
        item.status = BindingTargetStatus::Cancelled;
        assert!(
            item.move_to(BindingTargetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
