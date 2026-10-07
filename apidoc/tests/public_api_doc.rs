//! Writes docs/public-api/ for the parent package. Format: the zenutils-apidoc README.
#[test]
fn public_api_surface_docs_are_current() {
    zenutils_apidoc::ApiDoc::new().workspace_dir("..").run();
}
