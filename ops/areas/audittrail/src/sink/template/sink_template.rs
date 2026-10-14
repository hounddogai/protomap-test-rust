use super::*;

use acme_kit::text;

/// Where a sink template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SinkTemplateStatus {
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

impl std::fmt::Display for SinkTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink template as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkTemplate {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub ceiling: f64,
    pub tags: Vec<String>,
    pub expires_at: i64,
    pub status: SinkTemplateStatus,
}

impl SinkTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkTemplateStatus) -> bool {
        match self.status {
            SinkTemplateStatus::Open => {
                return next == SinkTemplateStatus::Reopened;
            }
            SinkTemplateStatus::InReview => {
                return next == SinkTemplateStatus::Resolved;
            }
            SinkTemplateStatus::Resolved => {
                match next {
                    SinkTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkTemplateStatus::Reopened => {
                return next == SinkTemplateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink template {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
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
        (self.ceiling * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkTemplate {
        SinkTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            ceiling: self.ceiling,
            tags: self.tags.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a sink template in status open.
pub fn new_sink_template(code: &str, title: &str) -> SinkTemplate {
    SinkTemplate {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        ceiling: 0.0,
        tags: Vec::new(),
        expires_at: 0,
        status: SinkTemplateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_sink_template("code-85", "title-439");
        assert_eq!(item.status, SinkTemplateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_sink_template("code-85", "title-439");
        if item.move_to(SinkTemplateStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkTemplateStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_sink_template("code-85", "title-439");
        item.status = SinkTemplateStatus::Archived;
        assert!(item.move_to(SinkTemplateStatus::Open).is_err(), "expected move to fail");
    }
}
