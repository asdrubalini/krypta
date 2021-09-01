use s3::creds::Credentials;
use s3::region::Region;
use s3::Bucket;

#[allow(dead_code)]

pub fn s3_authenticate() -> Bucket {
    let credentials = Credentials::new(
        Some("0004a4071d1e1350000000003"),
        Some("K000Rmju+zZNXAKa7mnj3rRGu5MLhuM"),
        None,
        None,
        None,
    )
    .unwrap();

    let region_name = "us-west-000".to_string();
    let endpoint = "https://s3.us-west-000.backblazeb2.com".to_string();
    let region = Region::Custom {
        region: region_name,
        endpoint: endpoint,
    };

    Bucket::new(
        "storage-a58b66e8b5eac514be2d806d46ab993ef04fcd9baa",
        region,
        credentials,
    )
    .unwrap()
}
