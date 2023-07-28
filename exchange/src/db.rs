use tonic::async_trait;

#[async_trait]
trait DbAccess {
    async fn dump() {}
    async fn get_tickers() {}
}

enum DbAction {}

mod s3db {
    struct S3Access {}
}
