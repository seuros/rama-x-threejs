use std::convert::Infallible;

use rama::futures::stream;
use rama::http::body::util::BodyExt as _;
use rama::http::service::web::response::{IntoResponse as _, Sse};

use super::*;

/// The bytes Sse puts on the wire for `event`.
async fn wire(event: DatastarEvent) -> String {
    let body = Sse::new(stream::iter([Ok::<_, Infallible>(event)]))
        .into_response()
        .into_body();
    let bytes = body.collect().await.expect("body").to_bytes();
    String::from_utf8(bytes.to_vec()).expect("utf8")
}

#[derive(Serialize)]
struct Floor {
    belts: Vec<&'static str>,
    at: u64,
}

#[tokio::test]
async fn encodes_one_patch_signals_event_with_the_state_as_json_text() {
    let floor = Floor {
        belts: vec!["default"],
        at: 7,
    };
    let event = SceneSignals::new("floor", &floor)
        .try_into_datastar_event()
        .expect("event");
    assert_eq!(
        wire(event).await,
        concat!(
            "event: datastar-patch-signals\n",
            r#"data: signals {"_three":{"floor":"{\"belts\":[\"default\"],\"at\":7}"}}"#,
            "\n\n",
        )
    );
}

#[tokio::test]
async fn only_if_missing_passes_through() {
    let event = DatastarEvent::try_from(SceneSignals::new("floor", &1).with_only_if_missing(true))
        .expect("event");
    assert!(wire(event).await.ends_with("data: onlyIfMissing true\n\n"));

    let mut signals = SceneSignals::new("floor", &1);
    signals.set_only_if_missing(true);
    assert!(
        signals
            .try_into_patch_signals()
            .expect("patch")
            .only_if_missing
    );
    assert!(
        !SceneSignals::new("floor", &1)
            .try_into_patch_signals()
            .expect("patch")
            .only_if_missing
    );
}

#[tokio::test]
async fn newlines_in_the_state_stay_on_one_data_line() {
    let event = SceneSignals::new("label", "two\nlines\r\n")
        .try_into_datastar_event()
        .expect("event");
    let wire = wire(event).await;
    assert_eq!(
        wire.lines().filter(|l| l.starts_with("data: ")).count(),
        1,
        "{wire}"
    );
}

#[test]
fn the_state_round_trips_through_its_json_text() {
    let state = serde_json::json!({"nodes": [1, 2], "edges": [[1, 2]]});
    let patch = SceneSignals::new("dag", &state)
        .try_into_patch_signals()
        .expect("patch");
    let signals: serde_json::Value = serde_json::from_str(&patch.signals).expect("JSON");
    let text = signals[NAMESPACE]["dag"].as_str().expect("JSON text");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(text).expect("JSON"),
        state
    );
}

#[test]
fn names_are_what_an_html_attribute_can_carry() {
    for good in ["floor", "dag2", "room_dag"] {
        assert!(
            SceneSignals::new(good, &1).try_into_patch_signals().is_ok(),
            "{good}"
        );
    }
    for bad in ["", "Floor", "2d", "_x", "a.b", "a-b", "a b", "é"] {
        let err = SceneSignals::new(bad, &1)
            .try_into_patch_signals()
            .expect_err(bad);
        assert!(err.to_string().contains("scene name"), "{err}");
    }
}

#[test]
fn a_state_that_cannot_serialize_is_an_error_not_a_panic() {
    struct Broken;
    impl Serialize for Broken {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("broken"))
        }
    }
    let err = SceneSignals::new("x", &Broken)
        .try_into_datastar_event()
        .expect_err("broken");
    assert!(err.to_string().contains("broken"), "{err}");
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn the_hook_reads_the_namespace_this_writes() {
    let hook = include_str!("../../assets/rama-x-threejs.js");
    assert!(hook.contains(&format!("const NAMESPACE = '{NAMESPACE}'")));
}
