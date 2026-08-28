use nav_studio_connector_lib::domain::ErrorCode;
use nav_studio_connector_lib::network::inspect_endpoint;
use std::io::Write;
use std::net::TcpListener;

#[test]
fn non_tls_endpoint_is_classified_without_accepting_it() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream.write_all(b"not tls").unwrap();
    });
    let error = inspect_endpoint(&format!("https://127.0.0.1:{}/", address.port()), 2).unwrap_err();
    server.join().unwrap();
    assert_eq!(error.code, ErrorCode::TlsFetchFailed);
}
