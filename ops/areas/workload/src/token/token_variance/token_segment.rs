use super::*;

use acme_kit::text;

/// Where a token segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenSegmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TokenSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TokenSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One token segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSegment {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub retries: i64,
    pub external_ref: String,
    pub channel: String,
    pub status: TokenSegmentStatus,
}

impl TokenSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenSegmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenSegmentStatus) -> bool {
        match self.status {
            TokenSegmentStatus::Requested => {
                match next {
                    TokenSegmentStatus::Granted | TokenSegmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenSegmentStatus::Granted => {
                return next == TokenSegmentStatus::Revoked;
            }
            TokenSegmentStatus::Revoked => {
                match next {
                    TokenSegmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: TokenSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "token segment", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.retries as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenSegment {
        TokenSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            retries: self.retries,
            external_ref: self.external_ref.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a token segment in status requested.
pub fn new_token_segment(code: &str, label: &str) -> TokenSegment {
    TokenSegment {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        retries: 0,
        external_ref: "".to_string(),
        channel: "".to_string(),
        status: TokenSegmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_token_segment("code-819", "label-311");
        assert_eq!(item.status, TokenSegmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_token_segment("code-819", "label-311");
        if item.move_to(TokenSegmentStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenSegmentStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_token_segment("code-819", "label-311");
        item.status = TokenSegmentStatus::Expired;
        assert!(
            item.move_to(TokenSegmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
