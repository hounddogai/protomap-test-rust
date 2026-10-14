use super::*;

use acme_kit::text;

/// Where a redaction variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RedactionVarianceStatus {
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

impl std::fmt::Display for RedactionVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One redaction variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionVariance {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub category: String,
    pub issuer: String,
    pub port: i64,
    pub budget: f64,
    pub tags: Vec<String>,
    pub source: String,
    pub external_ref: String,
    pub duration_minutes: i64,
    pub status: RedactionVarianceStatus,
}

impl RedactionVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionVarianceStatus) -> bool {
        match self.status {
            RedactionVarianceStatus::Planned => {
                return next == RedactionVarianceStatus::Cancelled;
            }
            RedactionVarianceStatus::Scheduled => {
                match next {
                    RedactionVarianceStatus::Completed | RedactionVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionVarianceStatus::InProgress => {
                match next {
                    RedactionVarianceStatus::Completed | RedactionVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionVarianceStatus::Completed => {
                match next {
                    RedactionVarianceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RedactionVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_total(&self) -> f64 {
        ((self.weight + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionVariance {
        RedactionVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            category: self.category.clone(),
            issuer: self.issuer.clone(),
            port: self.port,
            budget: self.budget,
            tags: self.tags.clone(),
            source: self.source.clone(),
            external_ref: self.external_ref.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a redaction variance in status planned.
pub fn new_redaction_variance(code: &str, label: &str) -> RedactionVariance {
    RedactionVariance {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        category: "".to_string(),
        issuer: "".to_string(),
        port: 0,
        budget: 0.0,
        tags: Vec::new(),
        source: "".to_string(),
        external_ref: "".to_string(),
        duration_minutes: 0,
        status: RedactionVarianceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_redaction_variance("code-1", "label-727");
        assert_eq!(item.status, RedactionVarianceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_redaction_variance("code-1", "label-727");
        if item.move_to(RedactionVarianceStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionVarianceStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_redaction_variance("code-1", "label-727");
        item.status = RedactionVarianceStatus::Cancelled;
        assert!(
            item.move_to(RedactionVarianceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
