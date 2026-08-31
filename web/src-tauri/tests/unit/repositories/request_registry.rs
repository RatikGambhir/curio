use super::*;

fn len(registry: &RequestRegistry) -> usize {
    registry.requests.lock().expect("in-flight lock").len()
}

#[test]
fn cancellation_removes_every_in_flight_request() {
    let registry = RequestRegistry::default();
    let first = CancellationToken::new();
    let second = CancellationToken::new();
    registry
        .register("first".to_owned(), first.clone())
        .expect("register first");
    registry
        .register("second".to_owned(), second.clone())
        .expect("register second");

    assert_eq!(len(&registry), 2);
    assert!(registry.cancel("first").expect("cancel first"));
    assert!(first.is_cancelled());
    assert_eq!(len(&registry), 1);
    registry.remove("second").expect("remove second");
    assert_eq!(len(&registry), 0);
    assert!(!registry.cancel("missing").expect("cancel missing"));
}
