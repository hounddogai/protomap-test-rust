use super::*;

use acme_kit::text;

/// Where a probe variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeVarianceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ProbeVarianceStatus {
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

impl std::fmt::Display for ProbeVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a probe variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeVariance {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub pinned: bool,
    pub port: i64,
    pub issuer: String,
    pub ttl_seconds: i64,
    pub owner_id: String,
    pub quantity: i64,
    pub checksum: String,
    pub status: ProbeVarianceStatus,
}

impl ProbeVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeVarianceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeVarianceStatus) -> bool {
        match self.status {
            ProbeVarianceStatus::Open => {
                return next == ProbeVarianceStatus::Archived;
            }
            ProbeVarianceStatus::InReview => {
                match next {
                    ProbeVarianceStatus::Resolved | ProbeVarianceStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeVarianceStatus::Resolved => {
                return next == ProbeVarianceStatus::Reopened;
            }
            ProbeVarianceStatus::Reopened => {
                return next == ProbeVarianceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.quantity as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeVariance {
        ProbeVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            pinned: self.pinned,
            port: self.port,
            issuer: self.issuer.clone(),
            ttl_seconds: self.ttl_seconds,
            owner_id: self.owner_id.clone(),
            quantity: self.quantity,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a probe variance in status open.
pub fn new_probe_variance(code: &str, label: &str) -> ProbeVariance {
    ProbeVariance {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        pinned: false,
        port: 0,
        issuer: "".to_string(),
        ttl_seconds: 0,
        owner_id: "".to_string(),
        quantity: 0,
        checksum: "".to_string(),
        status: ProbeVarianceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_probe_variance("code-738", "label-324");
        assert_eq!(item.status, ProbeVarianceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_probe_variance("code-738", "label-324");
        if item.move_to(ProbeVarianceStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeVarianceStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_probe_variance("code-738", "label-324");
        item.status = ProbeVarianceStatus::Archived;
        assert!(
            item.move_to(ProbeVarianceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
