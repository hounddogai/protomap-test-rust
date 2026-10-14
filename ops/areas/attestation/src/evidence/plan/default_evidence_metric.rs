use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default evidence metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultEvidenceMetricStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultEvidenceMetricStatus {
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

impl std::fmt::Display for DefaultEvidenceMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default evidence metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEvidenceMetric {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub quota_units: i64,
    pub host: String,
    pub namespace: String,
    pub weight: f64,
    pub attributes: BTreeMap<String, String>,
    pub cipher: String,
    pub issuer: String,
    pub country: String,
    pub status: DefaultEvidenceMetricStatus,
}

impl DefaultEvidenceMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultEvidenceMetricStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultEvidenceMetricStatus) -> bool {
        match self.status {
            DefaultEvidenceMetricStatus::New => {
                match next {
                    DefaultEvidenceMetricStatus::Done | DefaultEvidenceMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultEvidenceMetricStatus::Queued => {
                return next == DefaultEvidenceMetricStatus::Running;
            }
            DefaultEvidenceMetricStatus::Running => {
                match next {
                    DefaultEvidenceMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultEvidenceMetricStatus::Done => {
                return next == DefaultEvidenceMetricStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultEvidenceMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default evidence metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.weight + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultEvidenceMetric {
        DefaultEvidenceMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            quota_units: self.quota_units,
            host: self.host.clone(),
            namespace: self.namespace.clone(),
            weight: self.weight,
            attributes: self.attributes.clone(),
            cipher: self.cipher.clone(),
            issuer: self.issuer.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a default evidence metric in status new.
pub fn new_default_evidence_metric(code: &str, title: &str) -> DefaultEvidenceMetric {
    DefaultEvidenceMetric {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        quota_units: 0,
        host: "".to_string(),
        namespace: "".to_string(),
        weight: 0.0,
        attributes: BTreeMap::new(),
        cipher: "".to_string(),
        issuer: "".to_string(),
        country: "".to_string(),
        status: DefaultEvidenceMetricStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_default_evidence_metric("code-121", "title-432");
        assert_eq!(item.status, DefaultEvidenceMetricStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_default_evidence_metric("code-121", "title-432");
        if item.move_to(DefaultEvidenceMetricStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultEvidenceMetricStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_default_evidence_metric("code-121", "title-432");
        item.status = DefaultEvidenceMetricStatus::Failed;
        assert!(
            item.move_to(DefaultEvidenceMetricStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
