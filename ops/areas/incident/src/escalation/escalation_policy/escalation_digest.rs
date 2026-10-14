use super::*;

use acme_kit::text;

/// Where a escalation digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationDigestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EscalationDigestStatus {
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

impl std::fmt::Display for EscalationDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation digest as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationDigest {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub revision: i64,
    pub label: String,
    pub ttl_seconds: i64,
    pub score: f64,
    pub pinned: bool,
    pub status: EscalationDigestStatus,
}

impl EscalationDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationDigestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationDigestStatus) -> bool {
        match self.status {
            EscalationDigestStatus::New => {
                match next {
                    EscalationDigestStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationDigestStatus::Queued => {
                match next {
                    EscalationDigestStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationDigestStatus::Running => {
                match next {
                    EscalationDigestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationDigestStatus::Done => {
                match next {
                    EscalationDigestStatus::Failed => {
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

    pub fn move_to(&mut self, next: EscalationDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationDigest {
        EscalationDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            revision: self.revision,
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            score: self.score,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a escalation digest in status new.
pub fn new_escalation_digest(code: &str, title: &str) -> EscalationDigest {
    EscalationDigest {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        revision: 0,
        label: "".to_string(),
        ttl_seconds: 0,
        score: 0.0,
        pinned: false,
        status: EscalationDigestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_escalation_digest("code-17", "title-922");
        assert_eq!(item.status, EscalationDigestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_escalation_digest("code-17", "title-922");
        if item.move_to(EscalationDigestStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationDigestStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_escalation_digest("code-17", "title-922");
        item.status = EscalationDigestStatus::Failed;
        assert!(
            item.move_to(EscalationDigestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
