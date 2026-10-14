use super::*;

use acme_kit::text;

/// Where a default silence metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultSilenceMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultSilenceMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for DefaultSilenceMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default silence metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSilenceMetric {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub rotation_days: i64,
    pub label: String,
    pub principal: String,
    pub tenant_id: String,
    pub contact_email: String,
    pub archived: bool,
    pub tags: Vec<String>,
    pub status: DefaultSilenceMetricStatus,
}

impl DefaultSilenceMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultSilenceMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultSilenceMetricStatus) -> bool {
        match self.status {
            DefaultSilenceMetricStatus::Open => {
                match next {
                    DefaultSilenceMetricStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultSilenceMetricStatus::InReview => {
                match next {
                    DefaultSilenceMetricStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultSilenceMetricStatus::Resolved => {
                return next == DefaultSilenceMetricStatus::Reopened || next == DefaultSilenceMetricStatus::Archived;
            }
            DefaultSilenceMetricStatus::Reopened => {
                match next {
                    DefaultSilenceMetricStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultSilenceMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default silence metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
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

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> DefaultSilenceMetric {
        DefaultSilenceMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            rotation_days: self.rotation_days,
            label: self.label.clone(),
            principal: self.principal.clone(),
            tenant_id: self.tenant_id.clone(),
            contact_email: self.contact_email.clone(),
            archived: self.archived,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default silence metric in status open.
pub fn new_default_silence_metric(code: &str, title: &str) -> DefaultSilenceMetric {
    DefaultSilenceMetric {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        rotation_days: 0,
        label: "".to_string(),
        principal: "".to_string(),
        tenant_id: "".to_string(),
        contact_email: "".to_string(),
        archived: false,
        tags: Vec::new(),
        status: DefaultSilenceMetricStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_silence_metric("code-841", "title-550");
        assert_eq!(item.status, DefaultSilenceMetricStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_default_silence_metric("code-841", "title-550");
        if item.move_to(DefaultSilenceMetricStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultSilenceMetricStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_silence_metric("code-841", "title-550");
        item.status = DefaultSilenceMetricStatus::Archived;
        assert!(
            item.move_to(DefaultSilenceMetricStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
