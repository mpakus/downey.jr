use ps_core::updates::from_github_json;

fn release_json(tag: &str, url: &str) -> String {
    format!(r#"{{"tag_name":"{tag}","html_url":"{url}"}}"#)
}

#[test]
fn newer_github_tag_is_an_available_update() {
    let check = from_github_json(
        "0.2.1",
        &release_json(
            "v0.2.2",
            "https://github.com/mpakus/1537paperstreet/releases/tag/v0.2.2",
        ),
    )
    .expect("parse");
    assert!(check.available);
    assert_eq!(check.current, "0.2.1");
    assert_eq!(check.latest, "0.2.2");
    assert_eq!(
        check.release_url,
        "https://github.com/mpakus/1537paperstreet/releases/tag/v0.2.2"
    );
    assert_eq!(check.message, "Version 0.2.2 is available.");
}

#[test]
fn matching_or_older_tags_are_up_to_date() {
    let same = from_github_json(
        "0.2.1",
        &release_json(
            "v0.2.1",
            "https://github.com/mpakus/1537paperstreet/releases/tag/v0.2.1",
        ),
    )
    .expect("same");
    assert!(!same.available);
    assert!(same.release_url.is_empty());
    assert_eq!(same.message, "You're up to date (0.2.1).");

    let older = from_github_json(
        "0.3.0",
        &release_json(
            "0.2.9",
            "https://github.com/mpakus/1537paperstreet/releases/tag/0.2.9",
        ),
    )
    .expect("older");
    assert!(!older.available);
}

#[test]
fn rejects_a_non_github_release_url() {
    let check = from_github_json(
        "0.2.1",
        &release_json("v0.3.0", "https://evil.example/download"),
    )
    .expect("parse");
    assert!(check.available);
    assert_eq!(
        check.release_url,
        "https://github.com/mpakus/1537paperstreet/releases"
    );
}

#[test]
fn rejects_invalid_release_payloads() {
    assert!(from_github_json("0.2.1", "not-json").is_err());
    assert!(
        from_github_json(
            "0.2.1",
            r#"{"html_url":"https://github.com/mpakus/1537paperstreet/releases"}"#
        )
        .is_err()
    );
    assert!(
        from_github_json(
            "0.2.1",
            &release_json(
                "nightly",
                "https://github.com/mpakus/1537paperstreet/releases/tag/nightly"
            )
        )
        .is_err()
    );
}
