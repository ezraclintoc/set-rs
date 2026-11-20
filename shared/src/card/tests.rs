use super::*;

#[test]
fn missing_id_examples() {
    // 0, 1, 2 differ only in fill.
    assert_eq!(missing_id(0, 1), 2);
    assert_eq!(missing_id(0, 2), 1);
    assert_eq!(missing_id(1, 2), 0);
}

#[test]
fn missing_id_completes_every_pair() {
    for a in 0..81 {
        for b in 0..81 {
            if a == b {
                continue;
            }
            let c = missing_id(a, b);
            assert!(c != a && c != b, "{a} {b} -> {c}");
            assert!(is_set_ids([a, b, c]), "{a} {b} -> {c}");
            assert_eq!(c, missing_card(&Card::from_id(a), &Card::from_id(b)).id());
        }
    }
}
