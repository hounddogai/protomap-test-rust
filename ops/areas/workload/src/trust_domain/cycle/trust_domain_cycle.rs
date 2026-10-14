use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a trust domain cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainCycleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TrustDomainCycleStatus {
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

impl std::fmt::Display for TrustDomainCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One trust domain cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainCycle {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub updated_at: i64,
    pub sequence: i64,
    pub discount: f64,
    pub maximum: f64,
    pub priority: i64,
    pub status: TrustDomainCycleStatus,
}

impl TrustDomainCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainCycleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainCycleStatus) -> bool {
        match self.status {
            TrustDomainCycleStatus::New => {
                match next {
                    TrustDomainCycleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainCycleStatus::Queued => {
                match next {
                    TrustDomainCycleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainCycleStatus::Running => {
                match next {
                    TrustDomainCycleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainCycleStatus::Done => {
                return next == TrustDomainCycleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain cycle {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "trust domain cycle", self.code)
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

    pub fn projected_exposure(&self) -> f64 {
        (self.discount * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainCycle {
        TrustDomainCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            updated_at: self.updated_at,
            sequence: self.sequence,
            discount: self.discount,
            maximum: self.maximum,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a trust domain cycle in status new.
pub fn new_trust_domain_cycle(code: &str, label: &str) -> TrustDomainCycle {
    TrustDomainCycle {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        updated_at: 0,
        sequence: 0,
        discount: 0.0,
        maximum: 0.0,
        priority: 0,
        status: TrustDomainCycleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_trust_domain_cycle("code-482", "label-490");
        assert_eq!(item.status, TrustDomainCycleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_trust_domain_cycle("code-482", "label-490");
        if item.move_to(TrustDomainCycleStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainCycleStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_trust_domain_cycle("code-482", "label-490");
        item.status = TrustDomainCycleStatus::Failed;
        assert!(
            item.move_to(TrustDomainCycleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
