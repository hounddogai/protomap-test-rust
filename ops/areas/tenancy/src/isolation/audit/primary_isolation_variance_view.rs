use super::*;

/// How screens and exports show a primary isolation variance.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIsolationVarianceView {
    pub code: String,
    pub title: String,
    pub margin: String,
    pub issuer_text: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_primary_isolation_variance_view(item: &PrimaryIsolationVariance) -> PrimaryIsolationVarianceView {
    PrimaryIsolationVarianceView {
        code: item.code.clone(),
        title: item.title.clone(),
        margin: format!("{:.2}", item.margin),
        issuer_text: item.issuer.clone(),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_isolation_variance_views(items: &[PrimaryIsolationVariance]) -> Vec<PrimaryIsolationVarianceView> {
    let mut views: Vec<PrimaryIsolationVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_isolation_variance_view(item));
    }
    views
}

pub fn sum_primary_isolation_variance_view(view: &PrimaryIsolationVarianceView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_isolation_variance("code-880", "title-348");
        let view = to_primary_isolation_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_isolation_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
