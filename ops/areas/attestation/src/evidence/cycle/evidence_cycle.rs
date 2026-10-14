use super::*;

use acme_kit::text;

/// Where a evidence cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceCycleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EvidenceCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for EvidenceCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCycle {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub segment: String,
    pub priority: i64,
    pub issuer: String,
    pub key_id: String,
    pub score: f64,
    pub status: EvidenceCycleStatus,
}

impl EvidenceCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceCycleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceCycleStatus) -> bool {
        match self.status {
            EvidenceCycleStatus::Planned => {
                match next {
                    EvidenceCycleStatus::Completed | EvidenceCycleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceCycleStatus::Scheduled => {
                match next {
                    EvidenceCycleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceCycleStatus::InProgress => {
                match next {
                    EvidenceCycleStatus::Completed | EvidenceCycleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceCycleStatus::Completed => {
                match next {
                    EvidenceCycleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: EvidenceCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.score - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceCycle {
        EvidenceCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            segment: self.segment.clone(),
            priority: self.priority,
            issuer: self.issuer.clone(),
            key_id: self.key_id.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a evidence cycle in status planned.
pub fn new_evidence_cycle(code: &str, name: &str) -> EvidenceCycle {
    EvidenceCycle {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        segment: "".to_string(),
        priority: 0,
        issuer: "".to_string(),
        key_id: "".to_string(),
        score: 0.0,
        status: EvidenceCycleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_evidence_cycle("code-611", "name-85");
        assert_eq!(item.status, EvidenceCycleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_evidence_cycle("code-611", "name-85");
        if item.move_to(EvidenceCycleStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceCycleStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_evidence_cycle("code-611", "name-85");
        item.status = EvidenceCycleStatus::Cancelled;
        assert!(
            item.move_to(EvidenceCycleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
