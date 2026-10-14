use super::*;

use acme_kit::text;

/// Where a slo notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SloNoticeStatus {
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

impl std::fmt::Display for SloNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SloNotice {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub weight: f64,
    pub fingerprint: String,
    pub region: String,
    pub host: String,
    pub audience: String,
    pub batch_size: i64,
    pub reviewer: String,
    pub minimum: f64,
    pub status: SloNoticeStatus,
}

impl SloNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloNoticeStatus) -> bool {
        match self.status {
            SloNoticeStatus::Open => {
                match next {
                    SloNoticeStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloNoticeStatus::InReview => {
                return next == SloNoticeStatus::Resolved || next == SloNoticeStatus::Reopened;
            }
            SloNoticeStatus::Resolved => {
                match next {
                    SloNoticeStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloNoticeStatus::Reopened => {
                match next {
                    SloNoticeStatus::Archived => {
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

    pub fn move_to(&mut self, next: SloNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloNotice {
        SloNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            weight: self.weight,
            fingerprint: self.fingerprint.clone(),
            region: self.region.clone(),
            host: self.host.clone(),
            audience: self.audience.clone(),
            batch_size: self.batch_size,
            reviewer: self.reviewer.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a slo notice in status open.
pub fn new_slo_notice(code: &str, title: &str) -> SloNotice {
    SloNotice {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        weight: 0.0,
        fingerprint: "".to_string(),
        region: "".to_string(),
        host: "".to_string(),
        audience: "".to_string(),
        batch_size: 0,
        reviewer: "".to_string(),
        minimum: 0.0,
        status: SloNoticeStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_slo_notice("code-952", "title-120");
        assert_eq!(item.status, SloNoticeStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_slo_notice("code-952", "title-120");
        if item.move_to(SloNoticeStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloNoticeStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_slo_notice("code-952", "title-120");
        item.status = SloNoticeStatus::Archived;
        assert!(item.move_to(SloNoticeStatus::Open).is_err(), "expected move to fail");
    }
}
