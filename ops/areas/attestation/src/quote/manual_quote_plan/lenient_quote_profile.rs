use super::*;

use acme_kit::text;

/// Where a lenient quote profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientQuoteProfileStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientQuoteProfileStatus {
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

impl std::fmt::Display for LenientQuoteProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient quote profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuoteProfile {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub region: String,
    pub source: String,
    pub active: bool,
    pub duration_minutes: i64,
    pub visible: bool,
    pub ceiling: f64,
    pub expires_at: i64,
    pub status: LenientQuoteProfileStatus,
}

impl LenientQuoteProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientQuoteProfileStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientQuoteProfileStatus) -> bool {
        match self.status {
            LenientQuoteProfileStatus::Idle => {
                return next == LenientQuoteProfileStatus::Ready;
            }
            LenientQuoteProfileStatus::Warming => {
                match next {
                    LenientQuoteProfileStatus::Ready | LenientQuoteProfileStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientQuoteProfileStatus::Ready => {
                match next {
                    LenientQuoteProfileStatus::Draining | LenientQuoteProfileStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientQuoteProfileStatus::Draining => {
                return next == LenientQuoteProfileStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientQuoteProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient quote profile {} cannot move from {} to {}",
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
        format!("{} {}", "lenient quote profile", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.expires_at as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientQuoteProfile {
        LenientQuoteProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            region: self.region.clone(),
            source: self.source.clone(),
            active: self.active,
            duration_minutes: self.duration_minutes,
            visible: self.visible,
            ceiling: self.ceiling,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a lenient quote profile in status idle.
pub fn new_lenient_quote_profile(code: &str, title: &str) -> LenientQuoteProfile {
    LenientQuoteProfile {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        region: "".to_string(),
        source: "".to_string(),
        active: false,
        duration_minutes: 0,
        visible: false,
        ceiling: 0.0,
        expires_at: 0,
        status: LenientQuoteProfileStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_quote_profile("code-264", "title-658");
        assert_eq!(item.status, LenientQuoteProfileStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_lenient_quote_profile("code-264", "title-658");
        if item.move_to(LenientQuoteProfileStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientQuoteProfileStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_quote_profile("code-264", "title-658");
        item.status = LenientQuoteProfileStatus::Stopped;
        assert!(
            item.move_to(LenientQuoteProfileStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
