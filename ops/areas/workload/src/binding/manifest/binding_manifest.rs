use super::*;

use acme_kit::text;

/// Where a binding manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BindingManifestStatus {
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

impl std::fmt::Display for BindingManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingManifest {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub weight: f64,
    pub rate: f64,
    pub score: f64,
    pub issuer: String,
    pub phone: String,
    pub status: BindingManifestStatus,
}

impl BindingManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingManifestStatus) -> bool {
        match self.status {
            BindingManifestStatus::Idle => {
                match next {
                    BindingManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingManifestStatus::Warming => {
                return next == BindingManifestStatus::Ready || next == BindingManifestStatus::Stopped;
            }
            BindingManifestStatus::Ready => {
                return next == BindingManifestStatus::Stopped;
            }
            BindingManifestStatus::Draining => {
                match next {
                    BindingManifestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BindingManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "binding manifest", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_value(&self) -> f64 {
        ((self.score - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingManifest {
        BindingManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            weight: self.weight,
            rate: self.rate,
            score: self.score,
            issuer: self.issuer.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a binding manifest in status idle.
pub fn new_binding_manifest(code: &str, title: &str) -> BindingManifest {
    BindingManifest {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        weight: 0.0,
        rate: 0.0,
        score: 0.0,
        issuer: "".to_string(),
        phone: "".to_string(),
        status: BindingManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_binding_manifest("code-755", "title-938");
        assert_eq!(item.status, BindingManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_binding_manifest("code-755", "title-938");
        if item.move_to(BindingManifestStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingManifestStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_binding_manifest("code-755", "title-938");
        item.status = BindingManifestStatus::Stopped;
        assert!(
            item.move_to(BindingManifestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
