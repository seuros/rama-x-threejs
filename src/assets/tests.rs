use rama::Service as _;
use rama::http::body::util::BodyExt as _;
use rama::http::{Body, HeaderMap, Method, Request, StatusCode};

use super::*;

async fn body_of(res: Response) -> (StatusCode, HeaderMap, Vec<u8>) {
    let (parts, body) = res.into_parts();
    let bytes = body.collect().await.expect("body").to_bytes().to_vec();
    (parts.status, parts.headers, bytes)
}

fn assert_immutable_js(status: StatusCode, headers: &HeaderMap) {
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers[header::CONTENT_TYPE],
        "text/javascript; charset=utf-8"
    );
    assert_eq!(
        headers[header::CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );
    assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
}

#[tokio::test]
async fn three_module_is_immutable_javascript() {
    let (status, headers, body) = body_of(ThreeModule::new().into_response()).await;
    assert_immutable_js(status, &headers);
    assert_eq!(body, MODULE);
}

#[tokio::test]
async fn three_hook_is_immutable_javascript() {
    let (status, headers, body) = body_of(ThreeHook::new().into_response()).await;
    assert_immutable_js(status, &headers);
    let hook = String::from_utf8(body).expect("utf8");
    assert!(hook.contains("import { attribute, effect, root } from 'datastar'"));
    assert!(hook.contains("name: 'three'"));
}

#[tokio::test]
async fn every_addon_is_immutable_javascript() {
    for addon in ThreeAddon::ALL {
        let (status, headers, body) = body_of(addon.into_response()).await;
        assert_immutable_js(status, &headers);
        assert!(body.len() > 5_000, "{addon:?} looks truncated");
        assert!(
            std::str::from_utf8(&body)
                .expect("utf8")
                .contains("from 'three'"),
            "{addon:?} is not a three addon"
        );
    }
}

fn get(path: &str) -> Request {
    Request::builder()
        .method(Method::GET)
        .uri(path)
        .body(Body::empty())
        .expect("request")
}

#[tokio::test]
async fn with_three_mounts_every_file_under_the_bundle() {
    let router: Router = Router::new().with_three("/three/");
    let base = base("/three");
    assert!(base.starts_with("/three/") && base.len() == "/three/".len() + 17);

    let mut paths: Vec<String> = [MODULE_PATH, HOOK_PATH]
        .iter()
        .map(|p| format!("{base}{p}"))
        .collect();
    paths.extend(
        ThreeAddon::ALL
            .iter()
            .map(|a| format!("{base}{ADDONS_PATH}{}", a.path())),
    );
    for path in paths {
        let res = router.serve(get(&path)).await.expect("routed");
        let (status, headers, _) = body_of(res).await;
        assert_immutable_js(status, &headers);
    }

    // rama's own Datastar, so the hook's `import 'datastar'` resolves, and the
    // source map it advertises.
    for path in [
        format!("{base}{DATASTAR_PATH}"),
        format!("{base}{DATASTAR_PATH}.map"),
    ] {
        let res = router.serve(get(&path)).await.expect("routed");
        let (status, _, body) = body_of(res).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert!(body.len() > 1_000, "{path}");
    }

    for stale in [
        "/three/0000000000000000/three.module.min.js".to_owned(),
        format!("{base}three.core.min.js"),
        format!("{base}{ADDONS_PATH}loaders/OBJLoader.js"),
    ] {
        // The router answers an unmatched path with its NotFound error,
        // which the error handler layer renders as 404.
        let res = router.serve(get(&stale)).await;
        assert!(
            res.as_ref()
                .map_or(true, |r| r.status() == StatusCode::NOT_FOUND),
            "{stale} was served"
        );
    }
}

/// Each `from '...'` inside the vendored files names a file mounted next to
/// it, or `three`.
#[test]
fn every_import_of_the_vendored_files_resolves() {
    let served: Vec<String> = ThreeAddon::ALL
        .iter()
        .map(|a| format!("{ADDONS_PATH}{}", a.path()))
        .collect();
    for addon in ThreeAddon::ALL {
        let text = std::str::from_utf8(addon.bytes()).expect("utf8");
        for line in text.lines().map(str::trim) {
            let Some(target) = line
                .strip_prefix("} from '")
                .or_else(|| {
                    line.strip_prefix("import { ")
                        .and_then(|l| l.split(" from '").nth(1))
                })
                .and_then(|l| l.split('\'').next())
            else {
                continue;
            };
            if target == "three" {
                continue;
            }
            let from = format!("{ADDONS_PATH}{}", addon.path());
            let mut dir: Vec<&str> = from.split('/').collect();
            dir.pop();
            for segment in target.split('/') {
                match segment {
                    "." => {}
                    ".." => {
                        dir.pop();
                    }
                    other => dir.push(other),
                }
            }
            let resolved = dir.join("/");
            assert!(
                served.contains(&resolved),
                "{} imports {target}, which is not mounted ({resolved})",
                addon.path()
            );
        }
    }
}

#[test]
fn the_vendored_three_is_the_pinned_release() {
    assert_eq!(
        include_str!("../../assets/three/VERSION").trim(),
        crate::THREE_VERSION
    );
    let minor = crate::THREE_VERSION.split('.').nth(1).expect("minor");
    let module = std::str::from_utf8(MODULE).expect("utf8");
    assert!(
        module.contains(&format!("=\"{minor}\"")),
        "REVISION {minor} not found"
    );
    assert!(
        module.starts_with("/**\n * @license"),
        "three's license header is gone"
    );
    // One module: nothing left to import, three.core included.
    assert!(
        !module.contains("three.core"),
        "the module still imports three.core"
    );
}

#[test]
fn the_bundle_follows_the_content() {
    assert_eq!(BUNDLE.len(), 16);
    assert!(BUNDLE.chars().all(|c| c.is_ascii_hexdigit()));
    // FNV-1a's published test vector, so the hash cannot drift silently.
    assert_eq!(fnv1a(FNV_OFFSET, b"a"), 0xaf63_dc4c_8601_ec8c);
}
