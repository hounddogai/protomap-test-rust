use super::*;

use acme_kit::text;

/// Where a nonce policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NoncePolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl NoncePolicyStatus {
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

impl std::fmt::Display for NoncePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a nonce policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct NoncePolicy {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub checksum: String,
    pub name: String,
    pub segment: String,
    pub cost: f64,
    pub status: NoncePolicyStatus,
}

impl NoncePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NoncePolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NoncePolicyStatus) -> bool {
        match self.status {
            NoncePolicyStatus::New => {
                match next {
                    NoncePolicyStatus::Queued | NoncePolicyStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NoncePolicyStatus::Queued => {
                return next == NoncePolicyStatus::Done;
            }
            NoncePolicyStatus::Running => {
                return next == NoncePolicyStatus::Done || next == NoncePolicyStatus::Failed;
            }
            NoncePolicyStatus::Done => {
                match next {
                    NoncePolicyStatus::Failed => {
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

    pub fn move_to(&mut self, next: NoncePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.amount + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NoncePolicy {
        NoncePolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            checksum: self.checksum.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a nonce policy in status new.
pub fn new_nonce_policy(code: &str, title: &str) -> NoncePolicy {
    NoncePolicy {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        checksum: "".to_string(),
        name: "".to_string(),
        segment: "".to_string(),
        cost: 0.0,
        status: NoncePolicyStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_nonce_policy("code-469", "title-690");
        assert_eq!(item.status, NoncePolicyStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_nonce_policy("code-469", "title-690");
        if item.move_to(NoncePolicyStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NoncePolicyStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_nonce_policy("code-469", "title-690");
        item.status = NoncePolicyStatus::Failed;
        assert!(item.move_to(NoncePolicyStatus::New).is_err(), "expected move to fail");
    }
}
