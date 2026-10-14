use super::*;

use acme_kit::text;

/// Where a zone summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneSummaryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ZoneSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ZoneSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSummary {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub checksum: String,
    pub window_days: i64,
    pub archived: bool,
    pub source: String,
    pub external_ref: String,
    pub issuer: String,
    pub sequence: i64,
    pub expires_at: i64,
    pub status: ZoneSummaryStatus,
}

impl ZoneSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneSummaryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneSummaryStatus) -> bool {
        match self.status {
            ZoneSummaryStatus::Proposed => {
                match next {
                    ZoneSummaryStatus::Rejected | ZoneSummaryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneSummaryStatus::Approved => {
                match next {
                    ZoneSummaryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneSummaryStatus::Rejected => {
                match next {
                    ZoneSummaryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ZoneSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone summary {} cannot move from {} to {}",
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

    pub fn projected_exposure(&self) -> f64 {
        ((self.window_days as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneSummary {
        ZoneSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            checksum: self.checksum.clone(),
            window_days: self.window_days,
            archived: self.archived,
            source: self.source.clone(),
            external_ref: self.external_ref.clone(),
            issuer: self.issuer.clone(),
            sequence: self.sequence,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a zone summary in status proposed.
pub fn new_zone_summary(code: &str, label: &str) -> ZoneSummary {
    ZoneSummary {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        checksum: "".to_string(),
        window_days: 0,
        archived: false,
        source: "".to_string(),
        external_ref: "".to_string(),
        issuer: "".to_string(),
        sequence: 0,
        expires_at: 0,
        status: ZoneSummaryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_zone_summary("code-75", "label-880");
        assert_eq!(item.status, ZoneSummaryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_zone_summary("code-75", "label-880");
        if item.move_to(ZoneSummaryStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneSummaryStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_zone_summary("code-75", "label-880");
        item.status = ZoneSummaryStatus::Withdrawn;
        assert!(
            item.move_to(ZoneSummaryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
