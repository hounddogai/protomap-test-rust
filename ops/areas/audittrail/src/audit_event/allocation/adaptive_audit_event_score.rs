use super::*;

use acme_kit::text;

/// Where a adaptive audit event score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveAuditEventScoreStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveAuditEventScoreStatus {
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

impl std::fmt::Display for AdaptiveAuditEventScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive audit event score as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventScore {
    pub code: String,
    pub name: String,
    pub source: String,
    pub checksum: String,
    pub issuer: String,
    pub priority: i64,
    pub status: AdaptiveAuditEventScoreStatus,
}

impl AdaptiveAuditEventScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveAuditEventScoreStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveAuditEventScoreStatus) -> bool {
        match self.status {
            AdaptiveAuditEventScoreStatus::Idle => {
                match next {
                    AdaptiveAuditEventScoreStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAuditEventScoreStatus::Warming => {
                return next == AdaptiveAuditEventScoreStatus::Ready || next == AdaptiveAuditEventScoreStatus::Stopped;
            }
            AdaptiveAuditEventScoreStatus::Ready => {
                match next {
                    AdaptiveAuditEventScoreStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAuditEventScoreStatus::Draining => {
                match next {
                    AdaptiveAuditEventScoreStatus::Stopped => {
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

    pub fn move_to(&mut self, next: AdaptiveAuditEventScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive audit event score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive audit event score", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> AdaptiveAuditEventScore {
        AdaptiveAuditEventScore {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            checksum: self.checksum.clone(),
            issuer: self.issuer.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive audit event score in status idle.
pub fn new_adaptive_audit_event_score(code: &str, name: &str) -> AdaptiveAuditEventScore {
    AdaptiveAuditEventScore {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        checksum: "".to_string(),
        issuer: "".to_string(),
        priority: 0,
        status: AdaptiveAuditEventScoreStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_adaptive_audit_event_score("code-938", "name-532");
        assert_eq!(item.status, AdaptiveAuditEventScoreStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_adaptive_audit_event_score("code-938", "name-532");
        if item.move_to(AdaptiveAuditEventScoreStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveAuditEventScoreStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_adaptive_audit_event_score("code-938", "name-532");
        item.status = AdaptiveAuditEventScoreStatus::Stopped;
        assert!(
            item.move_to(AdaptiveAuditEventScoreStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
