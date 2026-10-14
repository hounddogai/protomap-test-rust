use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a release metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseMetricStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ReleaseMetricStatus {
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

impl std::fmt::Display for ReleaseMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a release metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMetric {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub country: String,
    pub description: String,
    pub contact_email: String,
    pub summary: String,
    pub name: String,
    pub threshold: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: ReleaseMetricStatus,
}

impl ReleaseMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseMetricStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseMetricStatus) -> bool {
        match self.status {
            ReleaseMetricStatus::New => {
                match next {
                    ReleaseMetricStatus::Running | ReleaseMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseMetricStatus::Queued => {
                match next {
                    ReleaseMetricStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseMetricStatus::Running => {
                match next {
                    ReleaseMetricStatus::Done | ReleaseMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseMetricStatus::Done => {
                match next {
                    ReleaseMetricStatus::Failed => {
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

    pub fn move_to(&mut self, next: ReleaseMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "release metric", self.code)
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

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> ReleaseMetric {
        ReleaseMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            country: self.country.clone(),
            description: self.description.clone(),
            contact_email: self.contact_email.clone(),
            summary: self.summary.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a release metric in status new.
pub fn new_release_metric(code: &str, label: &str) -> ReleaseMetric {
    ReleaseMetric {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        country: "".to_string(),
        description: "".to_string(),
        contact_email: "".to_string(),
        summary: "".to_string(),
        name: "".to_string(),
        threshold: 0.0,
        attributes: BTreeMap::new(),
        status: ReleaseMetricStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_release_metric("code-985", "label-844");
        assert_eq!(item.status, ReleaseMetricStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_release_metric("code-985", "label-844");
        if item.move_to(ReleaseMetricStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseMetricStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_release_metric("code-985", "label-844");
        item.status = ReleaseMetricStatus::Failed;
        assert!(item.move_to(ReleaseMetricStatus::New).is_err(), "expected move to fail");
    }
}
