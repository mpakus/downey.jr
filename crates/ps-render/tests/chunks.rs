use ps_render::render;

#[test]
fn groups_top_level_blocks_near_sixty_four_kibibytes() {
    let first = "a".repeat(40 * 1024);
    let second = "b".repeat(30 * 1024);
    let third = "c".repeat(1024);
    let html = render(&format!("{first}\n\n{second}\n\n{third}\n"));

    assert_eq!(html.matches("<section class=\"chunk\">").count(), 2);
    let boundary = html
        .match_indices("<section class=\"chunk\">")
        .nth(1)
        .map(|(index, _)| index)
        .expect("second chunk");
    assert!(html[..boundary].contains(&first));
    assert!(html[..boundary].contains(&second));
    assert!(!html[..boundary].contains(&third));
    assert!(html[boundary..].contains(&third));
}

#[test]
fn never_splits_one_oversized_top_level_block() {
    let oversized = "x".repeat(70 * 1024);
    let html = render(&format!("{oversized}\n\ntail\n"));

    assert_eq!(html.matches("<section class=\"chunk\">").count(), 2);
    let boundary = html
        .match_indices("<section class=\"chunk\">")
        .nth(1)
        .map(|(index, _)| index)
        .expect("second chunk");
    assert!(html[..boundary].contains(&oversized));
    assert!(html[boundary..].contains("<p>tail</p>"));
}

#[test]
fn chunks_blocks_that_have_no_text_payload() {
    let markdown = "![](image.png)\n\n".repeat(3_000);
    let html = render(&markdown);

    assert!(html.matches("<section class=\"chunk\">").count() > 1);
}
