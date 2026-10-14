use super::*;

use acme_kit::text;

/// Where a trust domain entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TrustDomainEntryStatus {
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

impl std::fmt::Display for TrustDomainEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainEntry {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub reviewer: String,
    pub port: i64,
    pub capacity: i64,
    pub status: TrustDomainEntryStatus,
}

impl TrustDomainEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainEntryStatus) -> bool {
        match self.status {
            TrustDomainEntryStatus::New => {
                match next {
                    TrustDomainEntryStatus::Running | TrustDomainEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainEntryStatus::Queued => {
                match next {
                    TrustDomainEntryStatus::Done | TrustDomainEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainEntryStatus::Running => {
                match next {
                    TrustDomainEntryStatus::Done | TrustDomainEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainEntryStatus::Done => {
                match next {
                    TrustDomainEntryStatus::Failed => {
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

    pub fn move_to(&mut self, next: TrustDomainEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.capacity as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainEntry {
        TrustDomainEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            reviewer: self.reviewer.clone(),
            port: self.port,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a trust domain entry in status new.
pub fn new_trust_domain_entry(code: &str, name: &str) -> TrustDomainEntry {
    TrustDomainEntry {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        reviewer: "".to_string(),
        port: 0,
        capacity: 0,
        status: TrustDomainEntryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_trust_domain_entry("code-207", "name-688");
        assert_eq!(item.status, TrustDomainEntryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_trust_domain_entry("code-207", "name-688");
        if item.move_to(TrustDomainEntryStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainEntryStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_trust_domain_entry("code-207", "name-688");
        item.status = TrustDomainEntryStatus::Failed;
        assert!(
            item.move_to(TrustDomainEntryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
