use super::*;

use acme_kit::text;

/// Where a canary manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryManifestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CanaryManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for CanaryManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary manifest as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryManifest {
    pub code: String,
    pub name: String,
    pub host: String,
    pub summary: String,
    pub threshold: f64,
    pub rate: f64,
    pub status: CanaryManifestStatus,
}

impl CanaryManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryManifestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryManifestStatus) -> bool {
        match self.status {
            CanaryManifestStatus::Draft => {
                match next {
                    CanaryManifestStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryManifestStatus::Pending => {
                return next == CanaryManifestStatus::Active || next == CanaryManifestStatus::Suspended;
            }
            CanaryManifestStatus::Active => {
                match next {
                    CanaryManifestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryManifestStatus::Suspended => {
                match next {
                    CanaryManifestStatus::Closed => {
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

    pub fn move_to(&mut self, next: CanaryManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "canary manifest", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        (self.rate * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryManifest {
        CanaryManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            summary: self.summary.clone(),
            threshold: self.threshold,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a canary manifest in status draft.
pub fn new_canary_manifest(code: &str, name: &str) -> CanaryManifest {
    CanaryManifest {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        summary: "".to_string(),
        threshold: 0.0,
        rate: 0.0,
        status: CanaryManifestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_canary_manifest("code-607", "name-695");
        assert_eq!(item.status, CanaryManifestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_canary_manifest("code-607", "name-695");
        if item.move_to(CanaryManifestStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryManifestStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_canary_manifest("code-607", "name-695");
        item.status = CanaryManifestStatus::Closed;
        assert!(
            item.move_to(CanaryManifestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
