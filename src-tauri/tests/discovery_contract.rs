use nav_studio_connector_lib::network::{discover_robots, DISCOVERY_SERVICE};

#[test]
fn discovery_contract_uses_the_normative_service_type() {
    assert_eq!(DISCOVERY_SERVICE, "_umec-nav._tcp.local.");
}

#[test]
fn bounded_discovery_completes_without_subnet_scanning() {
    let result = discover_robots(1);
    assert!(
        result.is_ok(),
        "an empty local network is a valid discovery result: {result:?}"
    );
}
