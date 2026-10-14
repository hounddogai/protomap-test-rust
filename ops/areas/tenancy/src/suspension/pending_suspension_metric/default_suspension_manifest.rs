use super::*;

use acme_kit::text;

/// Where a default suspension manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultSuspensionManifestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultSuspensionManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultSuspensionManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default suspension manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSuspensionManifest {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub grace_days: i64,
    pub margin: f64,
    pub audience: String,
    pub attempts: i64,
    pub approved: bool,
    pub retries: i64,
    pub status: DefaultSuspensionManifestStatus,
}

impl DefaultSuspensionManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultSuspensionManifestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultSuspensionManifestStatus) -> bool {
        match self.status {
            DefaultSuspensionManifestStatus::Proposed => {
                match next {
                    DefaultSuspensionManifestStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultSuspensionManifestStatus::Approved => {
                match next {
                    DefaultSuspensionManifestStatus::Rejected | DefaultSuspensionManifestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultSuspensionManifestStatus::Rejected => {
                match next {
                    DefaultSuspensionManifestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DefaultSuspensionManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default suspension manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.retries as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultSuspensionManifest {
        DefaultSuspensionManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            grace_days: self.grace_days,
            margin: self.margin,
            audience: self.audience.clone(),
            attempts: self.attempts,
            approved: self.approved,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a default suspension manifest in status proposed.
pub fn new_default_suspension_manifest(code: &str, name: &str) -> DefaultSuspensionManifest {
    DefaultSuspensionManifest {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        grace_days: 0,
        margin: 0.0,
        audience: "".to_string(),
        attempts: 0,
        approved: false,
        retries: 0,
        status: DefaultSuspensionManifestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_suspension_manifest("code-253", "name-64");
        assert_eq!(item.status, DefaultSuspensionManifestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_default_suspension_manifest("code-253", "name-64");
        if item.move_to(DefaultSuspensionManifestStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultSuspensionManifestStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_suspension_manifest("code-253", "name-64");
        item.status = DefaultSuspensionManifestStatus::Withdrawn;
        assert!(
            item.move_to(DefaultSuspensionManifestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
