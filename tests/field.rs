use ifascript::field::{FieldBuffer, FieldEngine, FieldEvent, compute_field_hash};

fn make_event(odu_id: u16) -> FieldEvent {
    FieldEvent::new(odu_id, 1, "0xabc".to_string(), serde_json::json!({}))
}

#[test]
fn buffer_push_and_len() {
    let mut buf = FieldBuffer::new(10);
    buf.push(make_event(1));
    buf.push(make_event(2));
    buf.push(make_event(3));
    assert_eq!(buf.len(), 3);
}

#[test]
fn buffer_evicts_oldest() {
    let mut buf = FieldBuffer::new(2);
    buf.push(make_event(1));
    buf.push(make_event(2));
    buf.push(make_event(3));
    assert_eq!(buf.len(), 2, "buffer should evict oldest and keep 2 events");
}

#[test]
fn buffer_drain_clears() {
    let mut buf = FieldBuffer::new(10);
    buf.push(make_event(1));
    buf.push(make_event(2));
    let _ = buf.drain();
    assert_eq!(buf.len(), 0, "drain should clear all events");
}

#[test]
fn hash_empty_is_deterministic() {
    let h1 = compute_field_hash(&[]);
    let h2 = compute_field_hash(&[]);
    assert_eq!(h1, h2, "empty hash should be deterministic");
}

#[test]
fn hash_differs_with_different_events() {
    let empty_hash = compute_field_hash(&[]);
    let one_event_hash = compute_field_hash(&[make_event(42)]);
    assert_ne!(empty_hash, one_event_hash, "hash with one event should differ from empty hash");
}

#[test]
fn engine_ingest_and_flush() {
    let mut engine = FieldEngine::new("test-field");
    engine.ingest(make_event(1));
    engine.ingest(make_event(2));
    let flushed = engine.flush();
    assert_eq!(flushed.len(), 2, "flush should return 2 events");
    let second_flush = engine.flush();
    assert_eq!(second_flush.len(), 0, "second flush should return 0 events");
}

#[test]
fn engine_current_hash_changes() {
    let mut engine = FieldEngine::new("test-field");
    let hash_before = engine.current_hash();
    engine.ingest(make_event(99));
    let hash_after = engine.current_hash();
    assert_ne!(hash_before, hash_after, "current_hash should change after ingesting an event");
}

#[test]
fn field_event_constructor() {
    let event = FieldEvent::new(0, 1, "0xabc".to_string(), serde_json::json!({}));
    assert_eq!(event.odu_id, 0);
}
