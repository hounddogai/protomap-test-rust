use super::*;

use acme_kit::text;

/// Where a fallback region pin summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRegionPinSummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackRegionPinSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackRegionPinSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback region pin summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinSummary {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub weight: f64,
    pub expires_at: i64,
    pub ttl_seconds: i64,
    pub country: String,
    pub ratio: f64,
    pub status: FallbackRegionPinSummaryStatus,
}

impl FallbackRegionPinSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRegionPinSummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRegionPinSummaryStatus) -> bool {
        match self.status {
            FallbackRegionPinSummaryStatus::Requested => {
                return next == FallbackRegionPinSummaryStatus::Granted
                    || next == FallbackRegionPinSummaryStatus::Revoked;
            }
            FallbackRegionPinSummaryStatus::Granted => {
                return next == FallbackRegionPinSummaryStatus::Revoked;
            }
            FallbackRegionPinSummaryStatus::Revoked => {
                match next {
                    FallbackRegionPinSummaryStatus::Expired => {
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

    pub fn move_to(&mut self, next: FallbackRegionPinSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback region pin summary {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRegionPinSummary {
        FallbackRegionPinSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            weight: self.weight,
            expires_at: self.expires_at,
            ttl_seconds: self.ttl_seconds,
            country: self.country.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a fallback region pin summary in status requested.
pub fn new_fallback_region_pin_summary(code: &str, title: &str) -> FallbackRegionPinSummary {
    FallbackRegionPinSummary {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        weight: 0.0,
        expires_at: 0,
        ttl_seconds: 0,
        country: "".to_string(),
        ratio: 0.0,
        status: FallbackRegionPinSummaryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_region_pin_summary("code-301", "title-659");
        assert_eq!(item.status, FallbackRegionPinSummaryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_fallback_region_pin_summary("code-301", "title-659");
        if item.move_to(FallbackRegionPinSummaryStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRegionPinSummaryStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_region_pin_summary("code-301", "title-659");
        item.status = FallbackRegionPinSummaryStatus::Expired;
        assert!(
            item.move_to(FallbackRegionPinSummaryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
