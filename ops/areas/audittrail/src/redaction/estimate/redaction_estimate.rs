use super::*;

use acme_kit::text;

/// Where a redaction estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionEstimateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RedactionEstimateStatus {
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

impl std::fmt::Display for RedactionEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionEstimate {
    pub code: String,
    pub title: String,
    pub category: String,
    pub audience: String,
    pub locale: String,
    pub reviewer: String,
    pub expires_at: i64,
    pub tags: Vec<String>,
    pub budget: f64,
    pub ttl_seconds: i64,
    pub status: RedactionEstimateStatus,
}

impl RedactionEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionEstimateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionEstimateStatus) -> bool {
        match self.status {
            RedactionEstimateStatus::Planned => {
                return next == RedactionEstimateStatus::Scheduled || next == RedactionEstimateStatus::Cancelled;
            }
            RedactionEstimateStatus::Scheduled => {
                return next == RedactionEstimateStatus::Cancelled;
            }
            RedactionEstimateStatus::InProgress => {
                return next == RedactionEstimateStatus::Cancelled;
            }
            RedactionEstimateStatus::Completed => {
                return next == RedactionEstimateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction estimate {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.expires_at as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionEstimate {
        RedactionEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            audience: self.audience.clone(),
            locale: self.locale.clone(),
            reviewer: self.reviewer.clone(),
            expires_at: self.expires_at,
            tags: self.tags.clone(),
            budget: self.budget,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a redaction estimate in status planned.
pub fn new_redaction_estimate(code: &str, title: &str) -> RedactionEstimate {
    RedactionEstimate {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        audience: "".to_string(),
        locale: "".to_string(),
        reviewer: "".to_string(),
        expires_at: 0,
        tags: Vec::new(),
        budget: 0.0,
        ttl_seconds: 0,
        status: RedactionEstimateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_redaction_estimate("code-567", "title-918");
        assert_eq!(item.status, RedactionEstimateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_redaction_estimate("code-567", "title-918");
        if item.move_to(RedactionEstimateStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionEstimateStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_redaction_estimate("code-567", "title-918");
        item.status = RedactionEstimateStatus::Cancelled;
        assert!(
            item.move_to(RedactionEstimateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
