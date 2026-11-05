//! A cube the server spins: three.js from the crate, the scene module from
//! the app, the angle over Datastar.
//!
//! ```sh
//! cargo run --example spinner
//! # open http://127.0.0.1:8080/
//! curl -sN http://127.0.0.1:8080/stream | head -2
//! ```

use std::sync::Arc;
use std::time::Duration;

use rama::Layer as _;
use rama::error::BoxError;
use rama::futures::async_stream::stream_fn;
use rama::http::layer::error_handling::ErrorHandlerLayer;
use rama::http::server::HttpServer;
use rama::http::service::web::Router;
use rama::http::service::web::response::{Html, IntoResponse, Script, Sse};
use rama::http::sse::server::KeepAlive;
use rama::rt::Executor;
use rama_x_threejs::{ImportMap, SceneSignals, ThreeRouterExt};
use serde::Serialize;

/// The app's scene module: all of its three.js code.
const SPINNER_JS: &str = r#"
import * as THREE from 'three'

let renderer, scene, camera, cube

export function mount(el, state) {
  renderer = new THREE.WebGLRenderer({ canvas: el, antialias: true })
  scene = new THREE.Scene()
  camera = new THREE.PerspectiveCamera(50, el.clientWidth / el.clientHeight, 0.1, 100)
  camera.position.set(2.5, 2, 3.5)
  camera.lookAt(0, 0, 0)
  scene.add(new THREE.HemisphereLight(0xffffff, 0x334455, 2.5))
  cube = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1), new THREE.MeshStandardMaterial())
  scene.add(cube)
  renderer.setSize(el.clientWidth, el.clientHeight, false)
  renderer.setAnimationLoop(() => renderer.render(scene, camera))
  if (state) update(state)
  return () => renderer.dispose()
}

export function update({ angle, color }) {
  cube.rotation.set(angle / 2, angle, 0)
  cube.material.color.set(color)
}
"#;

async fn page() -> impl IntoResponse {
    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>rama-x-threejs spinner</title>
{import_map}
<script type="module">import "rama-x-threejs"</script>
</head>
<body data-init="@get('/stream')" style="margin: 0">
<canvas data-three:spin="/spinner.js" style="display: block; width: 100vw; height: 100vh"></canvas>
</body>
</html>"#,
        import_map = ImportMap::new("/three"),
    ))
}

#[derive(Serialize)]
struct Spin {
    angle: f64,
    color: String,
}

async fn stream() -> impl IntoResponse {
    Sse::new(stream_fn(|mut yielder| async move {
        let mut tick = 0u32;
        loop {
            let spin = Spin {
                angle: f64::from(tick) * 0.05,
                color: format!("hsl({}, 80%, 60%)", (tick * 2) % 360),
            };
            let event = SceneSignals::new("spin", &spin).try_into_datastar_event();
            yielder.yield_item(event).await;
            tokio::time::sleep(Duration::from_millis(50)).await;
            tick = tick.wrapping_add(1);
        }
    }))
    .with_keep_alive(KeepAlive::new())
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let router = Router::new()
        .with_three("/three")
        .with_get("/", page)
        .with_get("/stream", stream)
        .with_get("/spinner.js", Script(SPINNER_JS));

    eprintln!("open http://127.0.0.1:8080/");
    HttpServer::auto(Executor::default())
        .listen(
            "127.0.0.1:8080",
            Arc::new(ErrorHandlerLayer::new().into_layer(router)),
        )
        .await
}
