use super::*;

use acme_kit::text;

/// Where a fallback page exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPageExceptionStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackPageExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackPageExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback page exception as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageException {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub weight: f64,
    pub category: String,
    pub score: f64,
    pub port: i64,
    pub starts_at: i64,
    pub region: String,
    pub status: FallbackPageExceptionStatus,
}

impl FallbackPageException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPageExceptionStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPageExceptionStatus) -> bool {
        match self.status {
            FallbackPageExceptionStatus::Requested => {
                return next == FallbackPageExceptionStatus::Expired;
            }
            FallbackPageExceptionStatus::Granted => {
                return next == FallbackPageExceptionStatus::Revoked || next == FallbackPageExceptionStatus::Expired;
            }
            FallbackPageExceptionStatus::Revoked => {
                return next == FallbackPageExceptionStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPageExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback page exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.starts_at as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPageException {
        FallbackPageException {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            weight: self.weight,
            category: self.category.clone(),
            score: self.score,
            port: self.port,
            starts_at: self.starts_at,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a fallback page exception in status requested.
pub fn new_fallback_page_exception(code: &str, label: &str) -> FallbackPageException {
    FallbackPageException {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        weight: 0.0,
        category: "".to_string(),
        score: 0.0,
        port: 0,
        starts_at: 0,
        region: "".to_string(),
        status: FallbackPageExceptionStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_page_exception("code-133", "label-327");
        assert_eq!(item.status, FallbackPageExceptionStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_fallback_page_exception("code-133", "label-327");
        if item.move_to(FallbackPageExceptionStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPageExceptionStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_page_exception("code-133", "label-327");
        item.status = FallbackPageExceptionStatus::Expired;
        assert!(
            item.move_to(FallbackPageExceptionStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
