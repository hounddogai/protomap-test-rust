use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a sink plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkPlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SinkPlanStatus {
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

impl std::fmt::Display for SinkPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink plan as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkPlan {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub host: String,
    pub attributes: BTreeMap<String, String>,
    pub retries: i64,
    pub cost: f64,
    pub status: SinkPlanStatus,
}

impl SinkPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkPlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkPlanStatus) -> bool {
        match self.status {
            SinkPlanStatus::Open => {
                match next {
                    SinkPlanStatus::Resolved | SinkPlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkPlanStatus::InReview => {
                return next == SinkPlanStatus::Resolved || next == SinkPlanStatus::Archived;
            }
            SinkPlanStatus::Resolved => {
                return next == SinkPlanStatus::Reopened || next == SinkPlanStatus::Archived;
            }
            SinkPlanStatus::Reopened => {
                match next {
                    SinkPlanStatus::Archived => {
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

    pub fn move_to(&mut self, next: SinkPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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
        ((self.cost + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkPlan {
        SinkPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            host: self.host.clone(),
            attributes: self.attributes.clone(),
            retries: self.retries,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a sink plan in status open.
pub fn new_sink_plan(code: &str, title: &str) -> SinkPlan {
    SinkPlan {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        host: "".to_string(),
        attributes: BTreeMap::new(),
        retries: 0,
        cost: 0.0,
        status: SinkPlanStatus::Open,
    }
}
