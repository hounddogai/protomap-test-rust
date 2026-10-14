use super::*;

use acme_kit::text;

/// Where a legacy postmortem manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyPostmortemManifestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyPostmortemManifestStatus {
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

impl std::fmt::Display for LegacyPostmortemManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy postmortem manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemManifest {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub margin: f64,
    pub source: String,
    pub ratio: f64,
    pub tags: Vec<String>,
    pub audience: String,
    pub approved: bool,
    pub status: LegacyPostmortemManifestStatus,
}

impl LegacyPostmortemManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyPostmortemManifestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyPostmortemManifestStatus) -> bool {
        match self.status {
            LegacyPostmortemManifestStatus::Draft => {
                return next == LegacyPostmortemManifestStatus::Suspended
                    || next == LegacyPostmortemManifestStatus::Closed;
            }
            LegacyPostmortemManifestStatus::Pending => {
                return next == LegacyPostmortemManifestStatus::Active
                    || next == LegacyPostmortemManifestStatus::Closed;
            }
            LegacyPostmortemManifestStatus::Active => {
                return next == LegacyPostmortemManifestStatus::Suspended
                    || next == LegacyPostmortemManifestStatus::Closed;
            }
            LegacyPostmortemManifestStatus::Suspended => {
                return next == LegacyPostmortemManifestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyPostmortemManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy postmortem manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_load(&self) -> f64 {
        ((self.ratio + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyPostmortemManifest {
        LegacyPostmortemManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            margin: self.margin,
            source: self.source.clone(),
            ratio: self.ratio,
            tags: self.tags.clone(),
            audience: self.audience.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a legacy postmortem manifest in status draft.
pub fn new_legacy_postmortem_manifest(code: &str, title: &str) -> LegacyPostmortemManifest {
    LegacyPostmortemManifest {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        margin: 0.0,
        source: "".to_string(),
        ratio: 0.0,
        tags: Vec::new(),
        audience: "".to_string(),
        approved: false,
        status: LegacyPostmortemManifestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_postmortem_manifest("code-712", "title-195");
        assert_eq!(item.status, LegacyPostmortemManifestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_legacy_postmortem_manifest("code-712", "title-195");
        if item.move_to(LegacyPostmortemManifestStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyPostmortemManifestStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_postmortem_manifest("code-712", "title-195");
        item.status = LegacyPostmortemManifestStatus::Closed;
        assert!(
            item.move_to(LegacyPostmortemManifestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
