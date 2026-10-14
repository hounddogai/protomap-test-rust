use super::*;

use acme_kit::text;

/// Where a manual probe notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualProbeNoticeStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualProbeNoticeStatus {
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

impl std::fmt::Display for ManualProbeNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual probe notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeNotice {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub floor_value: f64,
    pub amount: f64,
    pub ceiling: f64,
    pub tags: Vec<String>,
    pub status: ManualProbeNoticeStatus,
}

impl ManualProbeNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualProbeNoticeStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualProbeNoticeStatus) -> bool {
        match self.status {
            ManualProbeNoticeStatus::Idle => {
                return next == ManualProbeNoticeStatus::Ready || next == ManualProbeNoticeStatus::Stopped;
            }
            ManualProbeNoticeStatus::Warming => {
                match next {
                    ManualProbeNoticeStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualProbeNoticeStatus::Ready => {
                return next == ManualProbeNoticeStatus::Draining || next == ManualProbeNoticeStatus::Stopped;
            }
            ManualProbeNoticeStatus::Draining => {
                match next {
                    ManualProbeNoticeStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ManualProbeNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual probe notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual probe notice", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.cost - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualProbeNotice {
        ManualProbeNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            floor_value: self.floor_value,
            amount: self.amount,
            ceiling: self.ceiling,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual probe notice in status idle.
pub fn new_manual_probe_notice(code: &str, name: &str) -> ManualProbeNotice {
    ManualProbeNotice {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        floor_value: 0.0,
        amount: 0.0,
        ceiling: 0.0,
        tags: Vec::new(),
        status: ManualProbeNoticeStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_probe_notice("code-700", "name-255");
        assert_eq!(item.status, ManualProbeNoticeStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_manual_probe_notice("code-700", "name-255");
        if item.move_to(ManualProbeNoticeStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualProbeNoticeStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_probe_notice("code-700", "name-255");
        item.status = ManualProbeNoticeStatus::Stopped;
        assert!(
            item.move_to(ManualProbeNoticeStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
