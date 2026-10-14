use super::*;

use acme_kit::text;

/// Where a lenient token summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTokenSummaryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientTokenSummaryStatus {
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

impl std::fmt::Display for LenientTokenSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient token summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTokenSummary {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub floor_value: f64,
    pub tenant_id: String,
    pub namespace: String,
    pub starts_at: i64,
    pub source: String,
    pub status: LenientTokenSummaryStatus,
}

impl LenientTokenSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTokenSummaryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTokenSummaryStatus) -> bool {
        match self.status {
            LenientTokenSummaryStatus::Idle => {
                match next {
                    LenientTokenSummaryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTokenSummaryStatus::Warming => {
                match next {
                    LenientTokenSummaryStatus::Ready | LenientTokenSummaryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTokenSummaryStatus::Ready => {
                return next == LenientTokenSummaryStatus::Stopped;
            }
            LenientTokenSummaryStatus::Draining => {
                match next {
                    LenientTokenSummaryStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LenientTokenSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient token summary {} cannot move from {} to {}",
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.starts_at as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientTokenSummary {
        LenientTokenSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            floor_value: self.floor_value,
            tenant_id: self.tenant_id.clone(),
            namespace: self.namespace.clone(),
            starts_at: self.starts_at,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a lenient token summary in status idle.
pub fn new_lenient_token_summary(code: &str, label: &str) -> LenientTokenSummary {
    LenientTokenSummary {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        floor_value: 0.0,
        tenant_id: "".to_string(),
        namespace: "".to_string(),
        starts_at: 0,
        source: "".to_string(),
        status: LenientTokenSummaryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_token_summary("code-112", "label-429");
        assert_eq!(item.status, LenientTokenSummaryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_lenient_token_summary("code-112", "label-429");
        if item.move_to(LenientTokenSummaryStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientTokenSummaryStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_token_summary("code-112", "label-429");
        item.status = LenientTokenSummaryStatus::Stopped;
        assert!(
            item.move_to(LenientTokenSummaryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
