use super::*;

use acme_kit::text;

/// Where a evidence score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceScoreStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EvidenceScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for EvidenceScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScore {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub ceiling: f64,
    pub retries: i64,
    pub status: EvidenceScoreStatus,
}

impl EvidenceScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceScoreStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceScoreStatus) -> bool {
        match self.status {
            EvidenceScoreStatus::Idle => {
                match next {
                    EvidenceScoreStatus::Ready | EvidenceScoreStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceScoreStatus::Warming => {
                match next {
                    EvidenceScoreStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceScoreStatus::Ready => {
                match next {
                    EvidenceScoreStatus::Draining | EvidenceScoreStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceScoreStatus::Draining => {
                match next {
                    EvidenceScoreStatus::Stopped => {
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

    pub fn move_to(&mut self, next: EvidenceScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.retries as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceScore {
        EvidenceScore {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            ceiling: self.ceiling,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a evidence score in status idle.
pub fn new_evidence_score(code: &str, name: &str) -> EvidenceScore {
    EvidenceScore {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        ceiling: 0.0,
        retries: 0,
        status: EvidenceScoreStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_evidence_score("code-654", "name-159");
        assert_eq!(item.status, EvidenceScoreStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_evidence_score("code-654", "name-159");
        if item.move_to(EvidenceScoreStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceScoreStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_evidence_score("code-654", "name-159");
        item.status = EvidenceScoreStatus::Stopped;
        assert!(
            item.move_to(EvidenceScoreStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
