use s3::region::Region;
use s3::Bucket;
use s3::{
    creds::{AwsCredsError, Credentials},
    S3Error,
};

use crate::config::Endpoint;

pub enum AuthenticationError {
    CredentialsError(AwsCredsError),
    BucketError(S3Error),
}

pub fn s3_authenticate(endpoint: &Endpoint) -> Result<Bucket, AuthenticationError> {
    let credentials = Credentials::new(
        Some(&endpoint.access_key),
        Some(&endpoint.secret_key),
        None,
        None,
        None,
    )
    .map_err(AuthenticationError::CredentialsError)?;

    let region = Region::Custom {
        region: endpoint.region.clone(),
        endpoint: endpoint.endpoint_url.clone(),
    };

    let bucket = Bucket::new(&endpoint.bucket_name, region, credentials)
        .map_err(AuthenticationError::BucketError)?;

    Ok(bucket)
}
