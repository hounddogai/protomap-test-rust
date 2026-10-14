use super::*;

use acme_kit::text;

/// Where a manual token manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualTokenManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualTokenManifestStatus {
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

impl std::fmt::Display for ManualTokenManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual token manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTokenManifest {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub discount: f64,
    pub external_ref: String,
    pub priority: i64,
    pub phone: String,
    pub archived: bool,
    pub timezone: String,
    pub description: String,
    pub status: ManualTokenManifestStatus,
}

impl ManualTokenManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualTokenManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualTokenManifestStatus) -> bool {
        match self.status {
            ManualTokenManifestStatus::Idle => {
                match next {
                    ManualTokenManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTokenManifestStatus::Warming => {
                match next {
                    ManualTokenManifestStatus::Ready | ManualTokenManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTokenManifestStatus::Ready => {
                return next == ManualTokenManifestStatus::Draining || next == ManualTokenManifestStatus::Stopped;
            }
            ManualTokenManifestStatus::Draining => {
                match next {
                    ManualTokenManifestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ManualTokenManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual token manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual token manifest", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.attempts as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualTokenManifest {
        ManualTokenManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            discount: self.discount,
            external_ref: self.external_ref.clone(),
            priority: self.priority,
            phone: self.phone.clone(),
            archived: self.archived,
            timezone: self.timezone.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a manual token manifest in status idle.
pub fn new_manual_token_manifest(code: &str, label: &str) -> ManualTokenManifest {
    ManualTokenManifest {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        discount: 0.0,
        external_ref: "".to_string(),
        priority: 0,
        phone: "".to_string(),
        archived: false,
        timezone: "".to_string(),
        description: "".to_string(),
        status: ManualTokenManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_token_manifest("code-222", "label-741");
        assert_eq!(item.status, ManualTokenManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_manual_token_manifest("code-222", "label-741");
        if item.move_to(ManualTokenManifestStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualTokenManifestStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_token_manifest("code-222", "label-741");
        item.status = ManualTokenManifestStatus::Stopped;
        assert!(
            item.move_to(ManualTokenManifestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
