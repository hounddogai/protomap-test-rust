use super::*;

use acme_kit::text;

/// Where a peering profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringProfileStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PeeringProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PeeringProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringProfile {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub ttl_seconds: i64,
    pub region: String,
    pub status: PeeringProfileStatus,
}

impl PeeringProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringProfileStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringProfileStatus) -> bool {
        match self.status {
            PeeringProfileStatus::Created => {
                match next {
                    PeeringProfileStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringProfileStatus::Validated => {
                match next {
                    PeeringProfileStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringProfileStatus::Published => {
                match next {
                    PeeringProfileStatus::Retired => {
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

    pub fn move_to(&mut self, next: PeeringProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> PeeringProfile {
        PeeringProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            ttl_seconds: self.ttl_seconds,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a peering profile in status created.
pub fn new_peering_profile(code: &str, label: &str) -> PeeringProfile {
    PeeringProfile {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        ttl_seconds: 0,
        region: "".to_string(),
        status: PeeringProfileStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_peering_profile("code-245", "label-768");
        assert_eq!(item.status, PeeringProfileStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_peering_profile("code-245", "label-768");
        if item.move_to(PeeringProfileStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringProfileStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_peering_profile("code-245", "label-768");
        item.status = PeeringProfileStatus::Retired;
        assert!(
            item.move_to(PeeringProfileStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
