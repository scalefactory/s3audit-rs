# S3 Audit

S3 Audit is a tool that reports on various aspects of S3 buckets within an AWS
account.

For more information, read [We rewrote s3audit in Rust](https://scalefactory.com/blog/2021/10/27/we-rewrote-s3audit-in-rust/).

## Installation

```shell
cargo install s3audit
```

## Usage

AWS credentials will be taken from the environment, it is recommended to run
`s3audit` using a tool like [`aws-vault`].

```shell
# Report on all buckets
s3audit

# Report on all buckets with output in CSV format
s3audit --format=csv

# Enable only a few specific audits
s3audit --disable-check=all --enable-check=acl --enable-check=encryption

# Disable coloured output
env NO_COLOR=1 s3audit
```

### AWS permissions

You should use run `s3audit` as an IAM principal that is allowed to
call `s3:ListAllMyBuckets`, and is also allow to run these (read only)
actions for all buckets in your account:

- `s3:ListAllMyBuckets`
- `s3:GetBucketAcl`
- `s3:GetBucketLogging`
- `s3:GetBucketPolicy`
- `s3:GetBucketPublicAccessBlock`
- `s3:GetBucketVersioning`
- `s3:GetBucketWebsite`
- `s3:GetEncryptionConfiguration`

## Minimum Supported Rust Version (MSRV)

v1.78.0

## License

Licensed under either of

  * Apache License, Version 2.0
    ([LICENSE-APACHE] or https://www.apache.org/licenses/LICENSE-2.0)
  * MIT license
    ([LICENSE-MIT] or https://opensource.org/licenses/MIT)

at your option.


## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

<!-- links -->
[`aws-vault`]: https://github.com/99designs/aws-vault
[LICENSE-APACHE]: LICENSE-APACHE
[LICENSE-MIT]: LICENSE-MIT
