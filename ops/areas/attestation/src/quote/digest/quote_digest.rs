use super::*;

use acme_kit::text;

/// Where a quote digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl QuoteDigestStatus {
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

impl std::fmt::Display for QuoteDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One quote digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteDigest {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub unit: String,
    pub cluster: String,
    pub amount: f64,
    pub margin: f64,
    pub expires_at: i64,
    pub namespace: String,
    pub status: QuoteDigestStatus,
}

impl QuoteDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteDigestStatus) -> bool {
        match self.status {
            QuoteDigestStatus::Idle => {
                match next {
                    QuoteDigestStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteDigestStatus::Warming => {
                return next == QuoteDigestStatus::Draining;
            }
            QuoteDigestStatus::Ready => {
                return next == QuoteDigestStatus::Draining;
            }
            QuoteDigestStatus::Draining => {
                return next == QuoteDigestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote digest {} cannot move from {} to {}",
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
        format!("{} {}", "quote digest", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.margin - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteDigest {
        QuoteDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            unit: self.unit.clone(),
            cluster: self.cluster.clone(),
            amount: self.amount,
            margin: self.margin,
            expires_at: self.expires_at,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a quote digest in status idle.
pub fn new_quote_digest(code: &str, name: &str) -> QuoteDigest {
    QuoteDigest {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        unit: "".to_string(),
        cluster: "".to_string(),
        amount: 0.0,
        margin: 0.0,
        expires_at: 0,
        namespace: "".to_string(),
        status: QuoteDigestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_quote_digest("code-720", "name-195");
        assert_eq!(item.status, QuoteDigestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_quote_digest("code-720", "name-195");
        if item.move_to(QuoteDigestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteDigestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_quote_digest("code-720", "name-195");
        item.status = QuoteDigestStatus::Stopped;
        assert!(item.move_to(QuoteDigestStatus::Idle).is_err(), "expected move to fail");
    }
}
