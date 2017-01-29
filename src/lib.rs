//! This crate provides a zero-allocation, iterator/slice-based parser for extracting
//! [transfer encoding](https://tools.ietf.org/html/rfc7230#section-3.3.1) types as they
//! appear in the `Transfer-Encoding` request header. [Standard
//! encodings](http://www.iana.org/assignments/http-parameters/http-parameters.xhtml#transfer-coding)
//! are extracted as enum values, and unknown encodings are extracted as slices for
//! further processing.
//!
//! ## Example
//!
//! ```rust
//! use uhttp_transfer_encoding::{transfer_encodings, TransferEncoding, StdTransferEncoding};
//!
//! let mut encs = transfer_encodings(" gzip, custom-enc, chunked");
//! assert_eq!(encs.next(), Some(TransferEncoding::Std(StdTransferEncoding::Chunked)));
//! assert_eq!(encs.next(), Some(TransferEncoding::Other("custom-enc")));
//! assert_eq!(encs.next(), Some(TransferEncoding::Std(StdTransferEncoding::Gzip)));
//! assert_eq!(encs.next(), None);
//! ```

#![feature(conservative_impl_trait)]

use std::ascii::AsciiExt;

/// Create an iterator over transfer encoding layers from the given string in [the
/// form](https://tools.ietf.org/html/rfc7230#section-3.3.1) used by the
/// `Transfer-Encoding` header field.
///
/// Encodings are yielded in the order they must be decoded, with the outermost layer
/// yielded first and the innermost layer yielded last.
pub fn transfer_encodings<'a>(s: &'a str) -> impl Iterator<Item = TransferEncoding<'a>> {
    s.split(',').rev().map(TransferEncoding::new)
}

/// HTTP transfer encoding scheme.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum TransferEncoding<'a> {
    /// Standard defined scheme.
    Std(StdTransferEncoding),
    /// Unknown/nonstandard scheme with the contained name.
    ///
    /// The name is guaranteed to have no surrounding whitespace and requires
    /// case-insensitive comparison to other strings.
    Other(&'a str),
}

impl<'a> TransferEncoding<'a> {
    /// Parse a new `TransferEncoding` from the given string.
    pub fn new(s: &'a str) -> Self {
        let s = s.trim();

        match s.parse() {
            Ok(enc) => TransferEncoding::Std(enc),
            Err(_) => TransferEncoding::Other(s),
        }
    }
}

/// Standard transfer encoding scheme, as defined by
/// [IANA](http://www.iana.org/assignments/http-parameters/http-parameters.xhtml#transfer-coding).
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum StdTransferEncoding {
    /// Split into a series of chunks.
    Chunked,
    /// UNIX "compress" data format.
    Compress,
    /// Deflate compressed data format.
    Deflate,
    /// Gzip compressed data format.
    Gzip,
}

impl std::str::FromStr for StdTransferEncoding {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use self::StdTransferEncoding::*;

        // Names are case-insensitive [RFC7230§4].
        if s.eq_ignore_ascii_case("chunked") {
            Ok(Chunked)
        } else if s.eq_ignore_ascii_case("compress") {
            Ok(Compress)
        } else if s.eq_ignore_ascii_case("deflate") {
            Ok(Deflate)
        } else if s.eq_ignore_ascii_case("gzip") {
            Ok(Gzip)
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_te() {
        use self::TransferEncoding::*;
        use self::StdTransferEncoding::*;

        assert_eq!(TransferEncoding::new("chunked"), Std(Chunked));
        assert_eq!(TransferEncoding::new("chUNked"), Std(Chunked));
        assert_eq!(TransferEncoding::new("chun  ked"), Other("chun  ked"));
        assert_eq!(TransferEncoding::new("  chun\tked\t"), Other("chun\tked"));
        assert_eq!(TransferEncoding::new("CHUNKED"), Std(Chunked));
        assert_eq!(TransferEncoding::new("  CHUNKed "), Std(Chunked));
        assert_eq!(TransferEncoding::new("compress"), Std(Compress));
        assert_eq!(TransferEncoding::new(" cOMPress  "), Std(Compress));
        assert_eq!(TransferEncoding::new("deflate"), Std(Deflate));
        assert_eq!(TransferEncoding::new("\n\r dEflAte \r"), Std(Deflate));
        assert_eq!(TransferEncoding::new("gzip"), Std(Gzip));
        assert_eq!(TransferEncoding::new(" GZiP\r\r\t"), Std(Gzip));
        assert_eq!(TransferEncoding::new("\tgzIP  "), Std(Gzip));
        assert_eq!(TransferEncoding::new(""), Other(""));
        assert_eq!(TransferEncoding::new("    \t "), Other(""));
        assert_eq!(TransferEncoding::new("ÆØБД❤"), Other("ÆØБД❤"));
    }

    #[test]
    fn test_tes() {
        use self::TransferEncoding::*;
        use self::StdTransferEncoding::*;

        let mut te = transfer_encodings("chunked");
        assert_eq!(te.next().unwrap(), Std(Chunked));
        assert!(te.next().is_none());

        let mut te = transfer_encodings("chunked, gzip, compress");
        assert_eq!(te.next().unwrap(), Std(Compress));
        assert_eq!(te.next().unwrap(), Std(Gzip));
        assert_eq!(te.next().unwrap(), Std(Chunked));
        assert!(te.next().is_none());

        let mut te = transfer_encodings("  ChuNKEd    ,\t\t gZIp\r\r, coMPRess\n\t   ,       ,");
        assert_eq!(te.next().unwrap(), Other(""));
        assert_eq!(te.next().unwrap(), Other(""));
        assert_eq!(te.next().unwrap(), Std(Compress));
        assert_eq!(te.next().unwrap(), Std(Gzip));
        assert_eq!(te.next().unwrap(), Std(Chunked));
        assert!(te.next().is_none());

        let mut te = transfer_encodings("\t\tdeflate,hello,   UNknown\r\r");
        assert_eq!(te.next().unwrap(), Other("UNknown"));
        assert_eq!(te.next().unwrap(), Other("hello"));
        assert_eq!(te.next().unwrap(), Std(Deflate));
        assert!(te.next().is_none());
    }
}
