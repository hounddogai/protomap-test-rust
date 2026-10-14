use super::*;

use acme_kit::text;

/// Where a primary escalation group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryEscalationGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryEscalationGroupStatus {
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

impl std::fmt::Display for PrimaryEscalationGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary escalation group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationGroup {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub source: String,
    pub cost: f64,
    pub scope: String,
    pub active: bool,
    pub tenant_id: String,
    pub expires_at: i64,
    pub revision: i64,
    pub key_id: String,
    pub status: PrimaryEscalationGroupStatus,
}

impl PrimaryEscalationGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryEscalationGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryEscalationGroupStatus) -> bool {
        match self.status {
            PrimaryEscalationGroupStatus::Idle => {
                return next == PrimaryEscalationGroupStatus::Warming || next == PrimaryEscalationGroupStatus::Ready;
            }
            PrimaryEscalationGroupStatus::Warming => {
                match next {
                    PrimaryEscalationGroupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEscalationGroupStatus::Ready => {
                return next == PrimaryEscalationGroupStatus::Draining || next == PrimaryEscalationGroupStatus::Stopped;
            }
            PrimaryEscalationGroupStatus::Draining => {
                match next {
                    PrimaryEscalationGroupStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PrimaryEscalationGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary escalation group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.expires_at as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryEscalationGroup {
        PrimaryEscalationGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            source: self.source.clone(),
            cost: self.cost,
            scope: self.scope.clone(),
            active: self.active,
            tenant_id: self.tenant_id.clone(),
            expires_at: self.expires_at,
            revision: self.revision,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary escalation group in status idle.
pub fn new_primary_escalation_group(code: &str, name: &str) -> PrimaryEscalationGroup {
    PrimaryEscalationGroup {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        source: "".to_string(),
        cost: 0.0,
        scope: "".to_string(),
        active: false,
        tenant_id: "".to_string(),
        expires_at: 0,
        revision: 0,
        key_id: "".to_string(),
        status: PrimaryEscalationGroupStatus::Idle,
    }
}
