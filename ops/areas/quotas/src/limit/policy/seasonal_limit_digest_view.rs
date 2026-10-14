use super::*;

/// How screens and exports show a seasonal limit digest.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalLimitDigestView {
    pub code: String,
    pub title_text: String,
    pub checksum: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_seasonal_limit_digest_view(item: &SeasonalLimitDigest) -> SeasonalLimitDigestView {
    SeasonalLimitDigestView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        checksum: item.checksum.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_limit_digest_views(items: &[SeasonalLimitDigest]) -> Vec<SeasonalLimitDigestView> {
    let mut views: Vec<SeasonalLimitDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_limit_digest_view(item));
    }
    views
}

pub fn sum_seasonal_limit_digest_view(view: &SeasonalLimitDigestView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_limit_digest("code-317", "title-686");
        let view = to_seasonal_limit_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_limit_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
