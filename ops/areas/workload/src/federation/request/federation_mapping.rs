use super::*;

use acme_kit::text;

/// Where a federation mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationMappingStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FederationMappingStatus {
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

impl std::fmt::Display for FederationMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation mapping as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationMapping {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub weight: f64,
    pub category: String,
    pub limit: i64,
    pub key_id: String,
    pub notes: String,
    pub status: FederationMappingStatus,
}

impl FederationMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationMappingStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationMappingStatus) -> bool {
        match self.status {
            FederationMappingStatus::Idle => {
                match next {
                    FederationMappingStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationMappingStatus::Warming => {
                return next == FederationMappingStatus::Draining;
            }
            FederationMappingStatus::Ready => {
                match next {
                    FederationMappingStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationMappingStatus::Draining => {
                match next {
                    FederationMappingStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FederationMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation mapping {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "federation mapping", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.limit as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationMapping {
        FederationMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            weight: self.weight,
            category: self.category.clone(),
            limit: self.limit,
            key_id: self.key_id.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a federation mapping in status idle.
pub fn new_federation_mapping(code: &str, label: &str) -> FederationMapping {
    FederationMapping {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        weight: 0.0,
        category: "".to_string(),
        limit: 0,
        key_id: "".to_string(),
        notes: "".to_string(),
        status: FederationMappingStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_federation_mapping("code-674", "label-389");
        assert_eq!(item.status, FederationMappingStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_federation_mapping("code-674", "label-389");
        if item.move_to(FederationMappingStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationMappingStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_federation_mapping("code-674", "label-389");
        item.status = FederationMappingStatus::Stopped;
        assert!(
            item.move_to(FederationMappingStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
