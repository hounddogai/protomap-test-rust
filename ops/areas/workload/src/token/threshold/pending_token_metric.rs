use super::*;

use acme_kit::text;

/// Where a pending token metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingTokenMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingTokenMetricStatus {
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

impl std::fmt::Display for PendingTokenMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending token metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTokenMetric {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub contact_email: String,
    pub priority: i64,
    pub cipher: String,
    pub ttl_seconds: i64,
    pub updated_at: i64,
    pub issuer: String,
    pub checksum: String,
    pub starts_at: i64,
    pub status: PendingTokenMetricStatus,
}

impl PendingTokenMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingTokenMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingTokenMetricStatus) -> bool {
        match self.status {
            PendingTokenMetricStatus::Idle => {
                return next == PendingTokenMetricStatus::Warming;
            }
            PendingTokenMetricStatus::Warming => {
                return next == PendingTokenMetricStatus::Draining;
            }
            PendingTokenMetricStatus::Ready => {
                match next {
                    PendingTokenMetricStatus::Draining | PendingTokenMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingTokenMetricStatus::Draining => {
                match next {
                    PendingTokenMetricStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingTokenMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending token metric {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.updated_at as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingTokenMetric {
        PendingTokenMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            contact_email: self.contact_email.clone(),
            priority: self.priority,
            cipher: self.cipher.clone(),
            ttl_seconds: self.ttl_seconds,
            updated_at: self.updated_at,
            issuer: self.issuer.clone(),
            checksum: self.checksum.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending token metric in status idle.
pub fn new_pending_token_metric(code: &str, name: &str) -> PendingTokenMetric {
    PendingTokenMetric {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        contact_email: "".to_string(),
        priority: 0,
        cipher: "".to_string(),
        ttl_seconds: 0,
        updated_at: 0,
        issuer: "".to_string(),
        checksum: "".to_string(),
        starts_at: 0,
        status: PendingTokenMetricStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_token_metric("code-578", "name-299");
        assert_eq!(item.status, PendingTokenMetricStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_pending_token_metric("code-578", "name-299");
        if item.move_to(PendingTokenMetricStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingTokenMetricStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_token_metric("code-578", "name-299");
        item.status = PendingTokenMetricStatus::Stopped;
        assert!(
            item.move_to(PendingTokenMetricStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
