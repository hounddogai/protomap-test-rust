use super::*;

use acme_kit::text;

/// Where a evidence summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceSummaryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EvidenceSummaryStatus {
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

impl std::fmt::Display for EvidenceSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One evidence summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSummary {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub updated_at: i64,
    pub port: i64,
    pub cluster: String,
    pub duration_minutes: i64,
    pub phone: String,
    pub ends_at: i64,
    pub quantity: i64,
    pub status: EvidenceSummaryStatus,
}

impl EvidenceSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceSummaryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceSummaryStatus) -> bool {
        match self.status {
            EvidenceSummaryStatus::Open => {
                return next == EvidenceSummaryStatus::Reopened;
            }
            EvidenceSummaryStatus::InReview => {
                match next {
                    EvidenceSummaryStatus::Reopened | EvidenceSummaryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceSummaryStatus::Resolved => {
                return next == EvidenceSummaryStatus::Archived;
            }
            EvidenceSummaryStatus::Reopened => {
                match next {
                    EvidenceSummaryStatus::Archived => {
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

    pub fn move_to(&mut self, next: EvidenceSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.port as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceSummary {
        EvidenceSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            updated_at: self.updated_at,
            port: self.port,
            cluster: self.cluster.clone(),
            duration_minutes: self.duration_minutes,
            phone: self.phone.clone(),
            ends_at: self.ends_at,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a evidence summary in status open.
pub fn new_evidence_summary(code: &str, name: &str) -> EvidenceSummary {
    EvidenceSummary {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        updated_at: 0,
        port: 0,
        cluster: "".to_string(),
        duration_minutes: 0,
        phone: "".to_string(),
        ends_at: 0,
        quantity: 0,
        status: EvidenceSummaryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_evidence_summary("code-790", "name-698");
        assert_eq!(item.status, EvidenceSummaryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_evidence_summary("code-790", "name-698");
        if item.move_to(EvidenceSummaryStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceSummaryStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_evidence_summary("code-790", "name-698");
        item.status = EvidenceSummaryStatus::Archived;
        assert!(
            item.move_to(EvidenceSummaryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
