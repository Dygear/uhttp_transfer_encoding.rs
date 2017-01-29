# uhttp\_transfer\_encoding -- Parser for HTTP Transfer-Encoding header

[Documentation](https://docs.rs/uhttp_transfer_encoding)

This crate provides a zero-allocation, iterator/slice-based parser for extracting
[transfer encoding](https://tools.ietf.org/html/rfc7230#section-3.3.1) types as they
appear in the `Transfer-Encoding` request header. [Standard
encodings](http://www.iana.org/assignments/http-parameters/http-parameters.xhtml#transfer-coding)
are extracted as enum values, and unknown encodings are extracted as slices for
further processing.

## Example

```rust
use uhttp_transfer_encoding::{transfer_encodings, TransferEncoding, StdTransferEncoding};

let mut encs = transfer_encodings(" gzip, custom-enc, chunked");
assert_eq!(encs.next(), Some(TransferEncoding::Std(StdTransferEncoding::Chunked)));
assert_eq!(encs.next(), Some(TransferEncoding::Other("custom-enc")));
assert_eq!(encs.next(), Some(TransferEncoding::Std(StdTransferEncoding::Gzip)));
assert_eq!(encs.next(), None);
```

## Usage

This [crate](https://crates.io/crates/uhttp_transfer_encoding) can be used through cargo by
adding it as a dependency in `Cargo.toml`:

```toml
[dependencies]
uhttp_transfer_encoding = "0.5.0"
```
and importing it in the crate root:

```rust
extern crate uhttp_transfer_encoding;
```
