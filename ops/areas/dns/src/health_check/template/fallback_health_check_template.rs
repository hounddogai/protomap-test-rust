use super::*;

use acme_kit::text;

/// Where a fallback health check template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackHealthCheckTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackHealthCheckTemplateStatus {
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

impl std::fmt::Display for FallbackHealthCheckTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback health check template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackHealthCheckTemplate {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub cluster: String,
    pub discount: f64,
    pub retries: i64,
    pub minimum: f64,
    pub ratio: f64,
    pub status: FallbackHealthCheckTemplateStatus,
}

impl FallbackHealthCheckTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackHealthCheckTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackHealthCheckTemplateStatus) -> bool {
        match self.status {
            FallbackHealthCheckTemplateStatus::Planned => {
                return next == FallbackHealthCheckTemplateStatus::Scheduled
                    || next == FallbackHealthCheckTemplateStatus::Cancelled;
            }
            FallbackHealthCheckTemplateStatus::Scheduled => {
                return next == FallbackHealthCheckTemplateStatus::InProgress
                    || next == FallbackHealthCheckTemplateStatus::Cancelled;
            }
            FallbackHealthCheckTemplateStatus::InProgress => {
                match next {
                    FallbackHealthCheckTemplateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackHealthCheckTemplateStatus::Completed => {
                return next == FallbackHealthCheckTemplateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackHealthCheckTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback health check template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.minimum - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackHealthCheckTemplate {
        FallbackHealthCheckTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            cluster: self.cluster.clone(),
            discount: self.discount,
            retries: self.retries,
            minimum: self.minimum,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a fallback health check template in status planned.
pub fn new_fallback_health_check_template(code: &str, title: &str) -> FallbackHealthCheckTemplate {
    FallbackHealthCheckTemplate {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        cluster: "".to_string(),
        discount: 0.0,
        retries: 0,
        minimum: 0.0,
        ratio: 0.0,
        status: FallbackHealthCheckTemplateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_health_check_template("code-369", "title-983");
        assert_eq!(item.status, FallbackHealthCheckTemplateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_fallback_health_check_template("code-369", "title-983");
        if item.move_to(FallbackHealthCheckTemplateStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackHealthCheckTemplateStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_health_check_template("code-369", "title-983");
        item.status = FallbackHealthCheckTemplateStatus::Cancelled;
        assert!(
            item.move_to(FallbackHealthCheckTemplateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
