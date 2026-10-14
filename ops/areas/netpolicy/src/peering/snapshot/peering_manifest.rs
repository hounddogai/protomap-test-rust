use super::*;

use acme_kit::text;

/// Where a peering manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PeeringManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PeeringManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringManifest {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub title: String,
    pub cipher: String,
    pub attempts: i64,
    pub visible: bool,
    pub locale: String,
    pub status: PeeringManifestStatus,
}

impl PeeringManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringManifestStatus) -> bool {
        match self.status {
            PeeringManifestStatus::Planned => {
                match next {
                    PeeringManifestStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringManifestStatus::Scheduled => {
                match next {
                    PeeringManifestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringManifestStatus::InProgress => {
                return next == PeeringManifestStatus::Completed || next == PeeringManifestStatus::Cancelled;
            }
            PeeringManifestStatus::Completed => {
                match next {
                    PeeringManifestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PeeringManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "peering manifest", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.attempts as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringManifest {
        PeeringManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            attempts: self.attempts,
            visible: self.visible,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a peering manifest in status planned.
pub fn new_peering_manifest(code: &str, label: &str) -> PeeringManifest {
    PeeringManifest {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        title: "".to_string(),
        cipher: "".to_string(),
        attempts: 0,
        visible: false,
        locale: "".to_string(),
        status: PeeringManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_peering_manifest("code-361", "label-990");
        assert_eq!(item.status, PeeringManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_peering_manifest("code-361", "label-990");
        if item.move_to(PeeringManifestStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringManifestStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_peering_manifest("code-361", "label-990");
        item.status = PeeringManifestStatus::Cancelled;
        assert!(
            item.move_to(PeeringManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
