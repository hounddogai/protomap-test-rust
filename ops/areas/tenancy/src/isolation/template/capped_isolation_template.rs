use super::*;

use acme_kit::text;

/// Where a capped isolation template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedIsolationTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedIsolationTemplateStatus {
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

impl std::fmt::Display for CappedIsolationTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped isolation template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedIsolationTemplate {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub active: bool,
    pub capacity: i64,
    pub scope: String,
    pub status: CappedIsolationTemplateStatus,
}

impl CappedIsolationTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedIsolationTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedIsolationTemplateStatus) -> bool {
        match self.status {
            CappedIsolationTemplateStatus::Planned => {
                return next == CappedIsolationTemplateStatus::InProgress;
            }
            CappedIsolationTemplateStatus::Scheduled => {
                match next {
                    CappedIsolationTemplateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedIsolationTemplateStatus::InProgress => {
                return next == CappedIsolationTemplateStatus::Cancelled;
            }
            CappedIsolationTemplateStatus::Completed => {
                return next == CappedIsolationTemplateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedIsolationTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped isolation template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped isolation template", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> CappedIsolationTemplate {
        CappedIsolationTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            active: self.active,
            capacity: self.capacity,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a capped isolation template in status planned.
pub fn new_capped_isolation_template(code: &str, title: &str) -> CappedIsolationTemplate {
    CappedIsolationTemplate {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        active: false,
        capacity: 0,
        scope: "".to_string(),
        status: CappedIsolationTemplateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_capped_isolation_template("code-343", "title-939");
        assert_eq!(item.status, CappedIsolationTemplateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_capped_isolation_template("code-343", "title-939");
        if item.move_to(CappedIsolationTemplateStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedIsolationTemplateStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_capped_isolation_template("code-343", "title-939");
        item.status = CappedIsolationTemplateStatus::Cancelled;
        assert!(
            item.move_to(CappedIsolationTemplateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
