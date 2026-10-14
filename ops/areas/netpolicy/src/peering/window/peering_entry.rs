use super::*;

use acme_kit::text;

/// Where a peering entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PeeringEntryStatus {
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

impl std::fmt::Display for PeeringEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringEntry {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub channel: String,
    pub discount: f64,
    pub weight: f64,
    pub label: String,
    pub scope: String,
    pub status: PeeringEntryStatus,
}

impl PeeringEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringEntryStatus) -> bool {
        match self.status {
            PeeringEntryStatus::Idle => {
                match next {
                    PeeringEntryStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringEntryStatus::Warming => {
                match next {
                    PeeringEntryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringEntryStatus::Ready => {
                return next == PeeringEntryStatus::Stopped;
            }
            PeeringEntryStatus::Draining => {
                return next == PeeringEntryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "peering entry", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.discount + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringEntry {
        PeeringEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            channel: self.channel.clone(),
            discount: self.discount,
            weight: self.weight,
            label: self.label.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a peering entry in status idle.
pub fn new_peering_entry(code: &str, title: &str) -> PeeringEntry {
    PeeringEntry {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        channel: "".to_string(),
        discount: 0.0,
        weight: 0.0,
        label: "".to_string(),
        scope: "".to_string(),
        status: PeeringEntryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_peering_entry("code-529", "title-368");
        assert_eq!(item.status, PeeringEntryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_peering_entry("code-529", "title-368");
        if item.move_to(PeeringEntryStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringEntryStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_peering_entry("code-529", "title-368");
        item.status = PeeringEntryStatus::Stopped;
        assert!(item.move_to(PeeringEntryStatus::Idle).is_err(), "expected move to fail");
    }
}
