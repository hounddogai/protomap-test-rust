use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rolling page summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPageSummaryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingPageSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RollingPageSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling page summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageSummary {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub category: String,
    pub maximum: f64,
    pub tenant_id: String,
    pub attributes: BTreeMap<String, String>,
    pub attempts: i64,
    pub namespace: String,
    pub cipher: String,
    pub quota_units: i64,
    pub status: RollingPageSummaryStatus,
}

impl RollingPageSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPageSummaryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPageSummaryStatus) -> bool {
        match self.status {
            RollingPageSummaryStatus::New => {
                return next == RollingPageSummaryStatus::Failed;
            }
            RollingPageSummaryStatus::Queued => {
                return next == RollingPageSummaryStatus::Running;
            }
            RollingPageSummaryStatus::Running => {
                return next == RollingPageSummaryStatus::Failed;
            }
            RollingPageSummaryStatus::Done => {
                return next == RollingPageSummaryStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingPageSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling page summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn weighted_load(&self) -> f64 {
        ((self.attempts as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPageSummary {
        RollingPageSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            category: self.category.clone(),
            maximum: self.maximum,
            tenant_id: self.tenant_id.clone(),
            attributes: self.attributes.clone(),
            attempts: self.attempts,
            namespace: self.namespace.clone(),
            cipher: self.cipher.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rolling page summary in status new.
pub fn new_rolling_page_summary(code: &str, label: &str) -> RollingPageSummary {
    RollingPageSummary {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        category: "".to_string(),
        maximum: 0.0,
        tenant_id: "".to_string(),
        attributes: BTreeMap::new(),
        attempts: 0,
        namespace: "".to_string(),
        cipher: "".to_string(),
        quota_units: 0,
        status: RollingPageSummaryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_page_summary("code-400", "label-564");
        assert_eq!(item.status, RollingPageSummaryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_rolling_page_summary("code-400", "label-564");
        if item.move_to(RollingPageSummaryStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPageSummaryStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_page_summary("code-400", "label-564");
        item.status = RollingPageSummaryStatus::Failed;
        assert!(
            item.move_to(RollingPageSummaryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
