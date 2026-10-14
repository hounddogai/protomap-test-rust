use super::*;

use acme_kit::text;

/// Where a capped runbook variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRunbookVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedRunbookVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedRunbookVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped runbook variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRunbookVariance {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub starts_at: i64,
    pub description: String,
    pub ends_at: i64,
    pub status: CappedRunbookVarianceStatus,
}

impl CappedRunbookVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRunbookVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRunbookVarianceStatus) -> bool {
        match self.status {
            CappedRunbookVarianceStatus::Proposed => {
                match next {
                    CappedRunbookVarianceStatus::Approved | CappedRunbookVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRunbookVarianceStatus::Approved => {
                match next {
                    CappedRunbookVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRunbookVarianceStatus::Rejected => {
                match next {
                    CappedRunbookVarianceStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: CappedRunbookVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped runbook variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ends_at as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedRunbookVariance {
        CappedRunbookVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            starts_at: self.starts_at,
            description: self.description.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a capped runbook variance in status proposed.
pub fn new_capped_runbook_variance(code: &str, label: &str) -> CappedRunbookVariance {
    CappedRunbookVariance {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        starts_at: 0,
        description: "".to_string(),
        ends_at: 0,
        status: CappedRunbookVarianceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_runbook_variance("code-448", "label-874");
        assert_eq!(item.status, CappedRunbookVarianceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_capped_runbook_variance("code-448", "label-874");
        if item.move_to(CappedRunbookVarianceStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedRunbookVarianceStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_runbook_variance("code-448", "label-874");
        item.status = CappedRunbookVarianceStatus::Withdrawn;
        assert!(
            item.move_to(CappedRunbookVarianceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
