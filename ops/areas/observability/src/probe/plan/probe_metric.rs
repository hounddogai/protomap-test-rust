use super::*;

use acme_kit::text;

/// Where a probe metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ProbeMetricStatus {
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

impl std::fmt::Display for ProbeMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a probe metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMetric {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub quota_units: i64,
    pub notes: String,
    pub cost: f64,
    pub margin: f64,
    pub reviewer: String,
    pub cipher: String,
    pub status: ProbeMetricStatus,
}

impl ProbeMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeMetricStatus) -> bool {
        match self.status {
            ProbeMetricStatus::Open => {
                return next == ProbeMetricStatus::Reopened || next == ProbeMetricStatus::Archived;
            }
            ProbeMetricStatus::InReview => {
                match next {
                    ProbeMetricStatus::Resolved | ProbeMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeMetricStatus::Resolved => {
                return next == ProbeMetricStatus::Reopened || next == ProbeMetricStatus::Archived;
            }
            ProbeMetricStatus::Reopened => {
                match next {
                    ProbeMetricStatus::Archived => {
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

    pub fn move_to(&mut self, next: ProbeMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "probe metric", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.quota_units as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeMetric {
        ProbeMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            quota_units: self.quota_units,
            notes: self.notes.clone(),
            cost: self.cost,
            margin: self.margin,
            reviewer: self.reviewer.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a probe metric in status open.
pub fn new_probe_metric(code: &str, title: &str) -> ProbeMetric {
    ProbeMetric {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        quota_units: 0,
        notes: "".to_string(),
        cost: 0.0,
        margin: 0.0,
        reviewer: "".to_string(),
        cipher: "".to_string(),
        status: ProbeMetricStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_probe_metric("code-104", "title-935");
        assert_eq!(item.status, ProbeMetricStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_probe_metric("code-104", "title-935");
        if item.move_to(ProbeMetricStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeMetricStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_probe_metric("code-104", "title-935");
        item.status = ProbeMetricStatus::Archived;
        assert!(item.move_to(ProbeMetricStatus::Open).is_err(), "expected move to fail");
    }
}
