use super::*;

use acme_kit::text;

/// Where a trust domain preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainPreferenceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TrustDomainPreferenceStatus {
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

impl std::fmt::Display for TrustDomainPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a trust domain preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainPreference {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub cluster: String,
    pub priority: i64,
    pub unit: String,
    pub issuer: String,
    pub owner_id: String,
    pub key_id: String,
    pub visible: bool,
    pub status: TrustDomainPreferenceStatus,
}

impl TrustDomainPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainPreferenceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainPreferenceStatus) -> bool {
        match self.status {
            TrustDomainPreferenceStatus::New => {
                return next == TrustDomainPreferenceStatus::Done;
            }
            TrustDomainPreferenceStatus::Queued => {
                return next == TrustDomainPreferenceStatus::Running || next == TrustDomainPreferenceStatus::Done;
            }
            TrustDomainPreferenceStatus::Running => {
                match next {
                    TrustDomainPreferenceStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainPreferenceStatus::Done => {
                match next {
                    TrustDomainPreferenceStatus::Failed => {
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

    pub fn move_to(&mut self, next: TrustDomainPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ratio - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainPreference {
        TrustDomainPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            cluster: self.cluster.clone(),
            priority: self.priority,
            unit: self.unit.clone(),
            issuer: self.issuer.clone(),
            owner_id: self.owner_id.clone(),
            key_id: self.key_id.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a trust domain preference in status new.
pub fn new_trust_domain_preference(code: &str, name: &str) -> TrustDomainPreference {
    TrustDomainPreference {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        cluster: "".to_string(),
        priority: 0,
        unit: "".to_string(),
        issuer: "".to_string(),
        owner_id: "".to_string(),
        key_id: "".to_string(),
        visible: false,
        status: TrustDomainPreferenceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_trust_domain_preference("code-297", "name-331");
        assert_eq!(item.status, TrustDomainPreferenceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_trust_domain_preference("code-297", "name-331");
        if item.move_to(TrustDomainPreferenceStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainPreferenceStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_trust_domain_preference("code-297", "name-331");
        item.status = TrustDomainPreferenceStatus::Failed;
        assert!(
            item.move_to(TrustDomainPreferenceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
