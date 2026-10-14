use super::*;

use acme_kit::text;

/// Where a global escalation manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalEscalationManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalEscalationManifestStatus {
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

impl std::fmt::Display for GlobalEscalationManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global escalation manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEscalationManifest {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub unit: String,
    pub locale: String,
    pub duration_minutes: i64,
    pub capacity: i64,
    pub batch_size: i64,
    pub weight: f64,
    pub window_days: i64,
    pub owner_id: String,
    pub status: GlobalEscalationManifestStatus,
}

impl GlobalEscalationManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalEscalationManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalEscalationManifestStatus) -> bool {
        match self.status {
            GlobalEscalationManifestStatus::Idle => {
                match next {
                    GlobalEscalationManifestStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalEscalationManifestStatus::Warming => {
                match next {
                    GlobalEscalationManifestStatus::Ready | GlobalEscalationManifestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalEscalationManifestStatus::Ready => {
                match next {
                    GlobalEscalationManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalEscalationManifestStatus::Draining => {
                return next == GlobalEscalationManifestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalEscalationManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global escalation manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.capacity as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalEscalationManifest {
        GlobalEscalationManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            unit: self.unit.clone(),
            locale: self.locale.clone(),
            duration_minutes: self.duration_minutes,
            capacity: self.capacity,
            batch_size: self.batch_size,
            weight: self.weight,
            window_days: self.window_days,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a global escalation manifest in status idle.
pub fn new_global_escalation_manifest(code: &str, title: &str) -> GlobalEscalationManifest {
    GlobalEscalationManifest {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        unit: "".to_string(),
        locale: "".to_string(),
        duration_minutes: 0,
        capacity: 0,
        batch_size: 0,
        weight: 0.0,
        window_days: 0,
        owner_id: "".to_string(),
        status: GlobalEscalationManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_global_escalation_manifest("code-234", "title-87");
        assert_eq!(item.status, GlobalEscalationManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_global_escalation_manifest("code-234", "title-87");
        if item.move_to(GlobalEscalationManifestStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalEscalationManifestStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_global_escalation_manifest("code-234", "title-87");
        item.status = GlobalEscalationManifestStatus::Stopped;
        assert!(
            item.move_to(GlobalEscalationManifestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
