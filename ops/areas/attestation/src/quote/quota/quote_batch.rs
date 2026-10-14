use super::*;

use acme_kit::text;

/// Where a quote batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl QuoteBatchStatus {
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

impl std::fmt::Display for QuoteBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBatch {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub maximum: f64,
    pub visible: bool,
    pub status: QuoteBatchStatus,
}

impl QuoteBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteBatchStatus) -> bool {
        match self.status {
            QuoteBatchStatus::Idle => {
                match next {
                    QuoteBatchStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteBatchStatus::Warming => {
                return next == QuoteBatchStatus::Ready;
            }
            QuoteBatchStatus::Ready => {
                match next {
                    QuoteBatchStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteBatchStatus::Draining => {
                match next {
                    QuoteBatchStatus::Stopped => {
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

    pub fn move_to(&mut self, next: QuoteBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "quote batch", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> QuoteBatch {
        QuoteBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            maximum: self.maximum,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a quote batch in status idle.
pub fn new_quote_batch(code: &str, name: &str) -> QuoteBatch {
    QuoteBatch {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        maximum: 0.0,
        visible: false,
        status: QuoteBatchStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_quote_batch("code-574", "name-467");
        assert_eq!(item.status, QuoteBatchStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_quote_batch("code-574", "name-467");
        if item.move_to(QuoteBatchStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteBatchStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_quote_batch("code-574", "name-467");
        item.status = QuoteBatchStatus::Stopped;
        assert!(item.move_to(QuoteBatchStatus::Idle).is_err(), "expected move to fail");
    }
}
