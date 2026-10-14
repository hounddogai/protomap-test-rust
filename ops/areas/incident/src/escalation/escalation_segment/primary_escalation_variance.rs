use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary escalation variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryEscalationVarianceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryEscalationVarianceStatus {
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

impl std::fmt::Display for PrimaryEscalationVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary escalation variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationVariance {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub grace_days: i64,
    pub attributes: BTreeMap<String, String>,
    pub unit: String,
    pub tenant_id: String,
    pub status: PrimaryEscalationVarianceStatus,
}

impl PrimaryEscalationVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryEscalationVarianceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryEscalationVarianceStatus) -> bool {
        match self.status {
            PrimaryEscalationVarianceStatus::Idle => {
                match next {
                    PrimaryEscalationVarianceStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEscalationVarianceStatus::Warming => {
                match next {
                    PrimaryEscalationVarianceStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEscalationVarianceStatus::Ready => {
                return next == PrimaryEscalationVarianceStatus::Draining;
            }
            PrimaryEscalationVarianceStatus::Draining => {
                match next {
                    PrimaryEscalationVarianceStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PrimaryEscalationVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary escalation variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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

    pub fn effective_value(&self) -> f64 {
        ((self.port as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryEscalationVariance {
        PrimaryEscalationVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            grace_days: self.grace_days,
            attributes: self.attributes.clone(),
            unit: self.unit.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary escalation variance in status idle.
pub fn new_primary_escalation_variance(code: &str, title: &str) -> PrimaryEscalationVariance {
    PrimaryEscalationVariance {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        grace_days: 0,
        attributes: BTreeMap::new(),
        unit: "".to_string(),
        tenant_id: "".to_string(),
        status: PrimaryEscalationVarianceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_escalation_variance("code-23", "title-640");
        assert_eq!(item.status, PrimaryEscalationVarianceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_primary_escalation_variance("code-23", "title-640");
        if item.move_to(PrimaryEscalationVarianceStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryEscalationVarianceStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_escalation_variance("code-23", "title-640");
        item.status = PrimaryEscalationVarianceStatus::Stopped;
        assert!(
            item.move_to(PrimaryEscalationVarianceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
