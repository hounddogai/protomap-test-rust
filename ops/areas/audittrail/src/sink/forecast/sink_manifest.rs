use super::*;

use acme_kit::text;

/// Where a sink manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkManifestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SinkManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for SinkManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink manifest as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkManifest {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub ratio: f64,
    pub expires_at: i64,
    pub batch_size: i64,
    pub currency: String,
    pub quota_units: i64,
    pub source: String,
    pub status: SinkManifestStatus,
}

impl SinkManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkManifestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkManifestStatus) -> bool {
        match self.status {
            SinkManifestStatus::Open => {
                match next {
                    SinkManifestStatus::Resolved | SinkManifestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkManifestStatus::InReview => {
                match next {
                    SinkManifestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkManifestStatus::Resolved => {
                return next == SinkManifestStatus::Reopened || next == SinkManifestStatus::Archived;
            }
            SinkManifestStatus::Reopened => {
                match next {
                    SinkManifestStatus::Archived => {
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

    pub fn move_to(&mut self, next: SinkManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink manifest {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.expires_at as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkManifest {
        SinkManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            ratio: self.ratio,
            expires_at: self.expires_at,
            batch_size: self.batch_size,
            currency: self.currency.clone(),
            quota_units: self.quota_units,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a sink manifest in status open.
pub fn new_sink_manifest(code: &str, label: &str) -> SinkManifest {
    SinkManifest {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        ratio: 0.0,
        expires_at: 0,
        batch_size: 0,
        currency: "".to_string(),
        quota_units: 0,
        source: "".to_string(),
        status: SinkManifestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_sink_manifest("code-191", "label-453");
        assert_eq!(item.status, SinkManifestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_sink_manifest("code-191", "label-453");
        if item.move_to(SinkManifestStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkManifestStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_sink_manifest("code-191", "label-453");
        item.status = SinkManifestStatus::Archived;
        assert!(item.move_to(SinkManifestStatus::Open).is_err(), "expected move to fail");
    }
}
