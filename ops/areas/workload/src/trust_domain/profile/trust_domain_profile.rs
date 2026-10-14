use super::*;

use acme_kit::text;

/// Where a trust domain profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TrustDomainProfileStatus {
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

impl std::fmt::Display for TrustDomainProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain profile as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainProfile {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub sequence: i64,
    pub created_at: i64,
    pub external_ref: String,
    pub status: TrustDomainProfileStatus,
}

impl TrustDomainProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainProfileStatus) -> bool {
        match self.status {
            TrustDomainProfileStatus::New => {
                match next {
                    TrustDomainProfileStatus::Queued | TrustDomainProfileStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainProfileStatus::Queued => {
                match next {
                    TrustDomainProfileStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainProfileStatus::Running => {
                match next {
                    TrustDomainProfileStatus::Done | TrustDomainProfileStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainProfileStatus::Done => {
                return next == TrustDomainProfileStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain profile {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.priority as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainProfile {
        TrustDomainProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            sequence: self.sequence,
            created_at: self.created_at,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a trust domain profile in status new.
pub fn new_trust_domain_profile(code: &str, title: &str) -> TrustDomainProfile {
    TrustDomainProfile {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        sequence: 0,
        created_at: 0,
        external_ref: "".to_string(),
        status: TrustDomainProfileStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_trust_domain_profile("code-959", "title-441");
        assert_eq!(item.status, TrustDomainProfileStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_trust_domain_profile("code-959", "title-441");
        if item.move_to(TrustDomainProfileStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainProfileStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_trust_domain_profile("code-959", "title-441");
        item.status = TrustDomainProfileStatus::Failed;
        assert!(
            item.move_to(TrustDomainProfileStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
