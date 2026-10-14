use super::*;

use acme_kit::text;

/// Where a usage template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl UsageTemplateStatus {
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

impl std::fmt::Display for UsageTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageTemplate {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub version: i64,
    pub margin: f64,
    pub duration_minutes: i64,
    pub external_ref: String,
    pub status: UsageTemplateStatus,
}

impl UsageTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageTemplateStatus) -> bool {
        match self.status {
            UsageTemplateStatus::Planned => {
                return next == UsageTemplateStatus::InProgress || next == UsageTemplateStatus::Cancelled;
            }
            UsageTemplateStatus::Scheduled => {
                match next {
                    UsageTemplateStatus::InProgress | UsageTemplateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageTemplateStatus::InProgress => {
                match next {
                    UsageTemplateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageTemplateStatus::Completed => {
                return next == UsageTemplateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "usage template", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageTemplate {
        UsageTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            version: self.version,
            margin: self.margin,
            duration_minutes: self.duration_minutes,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a usage template in status planned.
pub fn new_usage_template(code: &str, label: &str) -> UsageTemplate {
    UsageTemplate {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        version: 0,
        margin: 0.0,
        duration_minutes: 0,
        external_ref: "".to_string(),
        status: UsageTemplateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_usage_template("code-922", "label-834");
        assert_eq!(item.status, UsageTemplateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_usage_template("code-922", "label-834");
        if item.move_to(UsageTemplateStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageTemplateStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_usage_template("code-922", "label-834");
        item.status = UsageTemplateStatus::Cancelled;
        assert!(
            item.move_to(UsageTemplateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
