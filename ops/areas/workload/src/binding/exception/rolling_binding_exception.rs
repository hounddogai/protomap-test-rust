use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rolling binding exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingBindingExceptionStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingBindingExceptionStatus {
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

impl std::fmt::Display for RollingBindingExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling binding exception as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingBindingException {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub retries: i64,
    pub summary: String,
    pub minimum: f64,
    pub attributes: BTreeMap<String, String>,
    pub tenant_id: String,
    pub created_at: i64,
    pub status: RollingBindingExceptionStatus,
}

impl RollingBindingException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingBindingExceptionStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingBindingExceptionStatus) -> bool {
        match self.status {
            RollingBindingExceptionStatus::Planned => {
                match next {
                    RollingBindingExceptionStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingBindingExceptionStatus::Scheduled => {
                match next {
                    RollingBindingExceptionStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingBindingExceptionStatus::InProgress => {
                return next == RollingBindingExceptionStatus::Completed
                    || next == RollingBindingExceptionStatus::Cancelled;
            }
            RollingBindingExceptionStatus::Completed => {
                return next == RollingBindingExceptionStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingBindingExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling binding exception {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.retries as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingBindingException {
        RollingBindingException {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            retries: self.retries,
            summary: self.summary.clone(),
            minimum: self.minimum,
            attributes: self.attributes.clone(),
            tenant_id: self.tenant_id.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rolling binding exception in status planned.
pub fn new_rolling_binding_exception(code: &str, label: &str) -> RollingBindingException {
    RollingBindingException {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        retries: 0,
        summary: "".to_string(),
        minimum: 0.0,
        attributes: BTreeMap::new(),
        tenant_id: "".to_string(),
        created_at: 0,
        status: RollingBindingExceptionStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_binding_exception("code-866", "label-625");
        assert_eq!(item.status, RollingBindingExceptionStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_rolling_binding_exception("code-866", "label-625");
        if item.move_to(RollingBindingExceptionStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingBindingExceptionStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_binding_exception("code-866", "label-625");
        item.status = RollingBindingExceptionStatus::Cancelled;
        assert!(
            item.move_to(RollingBindingExceptionStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
