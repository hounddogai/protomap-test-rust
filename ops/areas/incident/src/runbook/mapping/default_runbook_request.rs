use super::*;

use acme_kit::text;

/// Where a default runbook request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRunbookRequestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultRunbookRequestStatus {
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

impl std::fmt::Display for DefaultRunbookRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default runbook request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookRequest {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub namespace: String,
    pub description: String,
    pub attempts: i64,
    pub amount: f64,
    pub score: f64,
    pub retries: i64,
    pub ceiling: f64,
    pub country: String,
    pub status: DefaultRunbookRequestStatus,
}

impl DefaultRunbookRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRunbookRequestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRunbookRequestStatus) -> bool {
        match self.status {
            DefaultRunbookRequestStatus::Idle => {
                match next {
                    DefaultRunbookRequestStatus::Ready | DefaultRunbookRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRunbookRequestStatus::Warming => {
                match next {
                    DefaultRunbookRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRunbookRequestStatus::Ready => {
                return next == DefaultRunbookRequestStatus::Draining || next == DefaultRunbookRequestStatus::Stopped;
            }
            DefaultRunbookRequestStatus::Draining => {
                match next {
                    DefaultRunbookRequestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DefaultRunbookRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default runbook request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.retries as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRunbookRequest {
        DefaultRunbookRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            namespace: self.namespace.clone(),
            description: self.description.clone(),
            attempts: self.attempts,
            amount: self.amount,
            score: self.score,
            retries: self.retries,
            ceiling: self.ceiling,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a default runbook request in status idle.
pub fn new_default_runbook_request(code: &str, name: &str) -> DefaultRunbookRequest {
    DefaultRunbookRequest {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        namespace: "".to_string(),
        description: "".to_string(),
        attempts: 0,
        amount: 0.0,
        score: 0.0,
        retries: 0,
        ceiling: 0.0,
        country: "".to_string(),
        status: DefaultRunbookRequestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_runbook_request("code-655", "name-372");
        assert_eq!(item.status, DefaultRunbookRequestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_default_runbook_request("code-655", "name-372");
        if item.move_to(DefaultRunbookRequestStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRunbookRequestStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_runbook_request("code-655", "name-372");
        item.status = DefaultRunbookRequestStatus::Stopped;
        assert!(
            item.move_to(DefaultRunbookRequestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
