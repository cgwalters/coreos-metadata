use crate::providers::gcp;
use crate::providers::MetadataProvider;
use mockito;

#[test]
fn basic_hostname() {
    let ep = "/instance/hostname";
    let hostname = "test-hostname";

    let mut provider = gcp::GcpProvider::try_new().unwrap();
    provider.client = provider.client.max_attempts(1);

    let _m = mockito::mock("GET", ep).with_status(503).create();
    provider.hostname().unwrap_err();

    let _m = mockito::mock("GET", ep)
        .with_status(200)
        .with_body(hostname)
        .create();
    let v = provider.hostname().unwrap();
    assert_eq!(v, Some(hostname.to_string()));

    let _m = mockito::mock("GET", ep).with_status(404).create();
    let v = provider.hostname().unwrap();
    assert_eq!(v, None);

    mockito::reset();
    provider.hostname().unwrap_err();
}

#[test]
fn basic_attributes() {
    let hostname = "test-hostname";
    let ip_external = "test-ip-external";
    let ip_local = "test-ip-local";
    let machine_type = "test-machine-type";

    let endpoints = maplit::btreemap! {
        "/instance/hostname" => hostname,
        "/instance/network-interfaces/0/access-configs/0/external-ip" => ip_external,
        "/instance/network-interfaces/0/ip" => ip_local,
        "/instance/machine-type" => machine_type,
    };
    let mut mocks = Vec::with_capacity(endpoints.len());
    for (endpoint, body) in endpoints {
        let m = mockito::mock("GET", endpoint)
            .with_status(200)
            .with_body(body)
            .create();
        mocks.push(m);
    }

    let attributes = maplit::hashmap! {
        format!("{}_HOSTNAME", gcp::ENV_PREFIX) => hostname.to_string(),
        format!("{}_IP_EXTERNAL_0", gcp::ENV_PREFIX) => ip_external.to_string(),
        format!("{}_IP_LOCAL_0", gcp::ENV_PREFIX) => ip_local.to_string(),
        format!("{}_MACHINE_TYPE", gcp::ENV_PREFIX) => machine_type.to_string(),
    };

    let client = crate::retry::Client::try_new()
        .unwrap()
        .max_attempts(1)
        .return_on_404(true);
    let provider = gcp::GcpProvider { client };

    let v = provider.attributes().unwrap();
    assert_eq!(v, attributes);

    mockito::reset();
    provider.attributes().unwrap_err();
}
