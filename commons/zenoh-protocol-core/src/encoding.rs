use crate::ZInt;
use core::fmt;
use std::{borrow::Cow, convert::TryFrom};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnownEncoding {
    Empty = 0,
    AppOctetStream = 1,
    AppCustom = 2,
    TextPlain = 3,
    AppProperties = 4,
    AppJson = 5,
    AppSql = 6,
    AppInteger = 7,
    AppFloat = 8,
    AppXml = 9,
    AppXhtmlXml = 10,
    AppXWwwFormUrlencoded = 11,
    TextJson = 12,
    TextHtml = 13,
    TextXml = 14,
    TextCss = 15,
    TextCsv = 16,
    TextJavascript = 17,
    ImageJpeg = 18,
    ImagePng = 19,
    ImageGif = 20,
}
impl From<KnownEncoding> for u8 {
    fn from(val: KnownEncoding) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}
impl From<KnownEncoding> for usize {
    fn from(val: KnownEncoding) -> Self {
        u8::from(val) as usize
    }
}
impl std::convert::TryFrom<u8> for KnownEncoding {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value < consts::MIMES.len() as u8 + 1 {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(())
        }
    }
}
impl std::convert::TryFrom<ZInt> for KnownEncoding {
    type Error = ();
    fn try_from(value: ZInt) -> Result<Self, Self::Error> {
        if value < consts::MIMES.len() as ZInt + 1 {
            Ok(unsafe { std::mem::transmute(value as u8) })
        } else {
            Err(())
        }
    }
}
impl AsRef<str> for KnownEncoding {
    fn as_ref(&self) -> &str {
        consts::MIMES[usize::from(*self)]
    }
}
/// The encoding of a zenoh `zenoh::Value`.
///
/// A zenoh encoding is a HTTP Mime type represented, for wire efficiency,
/// as an integer prefix (that maps to a string) and a string suffix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Encoding {
    Exact(KnownEncoding),
    WithSuffix(KnownEncoding, Cow<'static, str>),
}

mod consts {
    pub(super) const MIMES: [&str; 21] = [
        /*  0 */ "",
        /*  1 */ "application/octet-stream",
        /*  2 */ "application/custom", // non iana standard
        /*  3 */ "text/plain",
        /*  4 */ "application/properties", // non iana standard
        /*  5 */ "application/json", // if not readable from casual users
        /*  6 */ "application/sql",
        /*  7 */ "application/integer", // non iana standard
        /*  8 */ "application/float", // non iana standard
        /*  9 */
        "application/xml", // if not readable from casual users (RFC 3023, section 3)
        /* 10 */ "application/xhtml+xml",
        /* 11 */ "application/x-www-form-urlencoded",
        /* 12 */ "text/json", // non iana standard - if readable from casual users
        /* 13 */ "text/html",
        /* 14 */ "text/xml", // if readable from casual users (RFC 3023, section 3)
        /* 15 */ "text/css",
        /* 16 */ "text/csv",
        /* 17 */ "text/javascript",
        /* 18 */ "image/jpeg",
        /* 19 */ "image/png",
        /* 20 */ "image/gif",
    ];
}

impl Encoding {
    pub fn new<IntoCowStr>(prefix: ZInt, suffix: IntoCowStr) -> Option<Self>
    where
        IntoCowStr: Into<Cow<'static, str>> + AsRef<str>,
    {
        let prefix = KnownEncoding::try_from(prefix).ok()?;
        if suffix.as_ref().is_empty() {
            Some(Encoding::Exact(prefix))
        } else {
            Some(Encoding::WithSuffix(prefix, suffix.into()))
        }
    }

    pub const EMPTY: Encoding = Encoding::Exact(KnownEncoding::Empty);
    pub const APP_OCTET_STREAM: Encoding = Encoding::Exact(KnownEncoding::AppOctetStream);
    pub const APP_CUSTOM: Encoding = Encoding::Exact(KnownEncoding::AppCustom);
    pub const TEXT_PLAIN: Encoding = Encoding::Exact(KnownEncoding::TextPlain);
    pub const STRING: Encoding = Encoding::TEXT_PLAIN;
    pub const APP_PROPERTIES: Encoding = Encoding::Exact(KnownEncoding::AppProperties);
    pub const APP_JSON: Encoding = Encoding::Exact(KnownEncoding::AppJson);
    pub const APP_SQL: Encoding = Encoding::Exact(KnownEncoding::AppSql);
    pub const APP_INTEGER: Encoding = Encoding::Exact(KnownEncoding::AppInteger);
    pub const APP_FLOAT: Encoding = Encoding::Exact(KnownEncoding::AppFloat);
    pub const APP_XML: Encoding = Encoding::Exact(KnownEncoding::AppXml);
    pub const APP_XHTML_XML: Encoding = Encoding::Exact(KnownEncoding::AppXhtmlXml);
    pub const APP_X_WWW_FORM_URLENCODED: Encoding =
        Encoding::Exact(KnownEncoding::AppXWwwFormUrlencoded);
    pub const TEXT_JSON: Encoding = Encoding::Exact(KnownEncoding::TextJson);
    pub const TEXT_HTML: Encoding = Encoding::Exact(KnownEncoding::TextHtml);
    pub const TEXT_XML: Encoding = Encoding::Exact(KnownEncoding::TextXml);
    pub const TEXT_CSS: Encoding = Encoding::Exact(KnownEncoding::TextCss);
    pub const TEXT_CSV: Encoding = Encoding::Exact(KnownEncoding::TextCsv);
    pub const TEXT_JAVASCRIPT: Encoding = Encoding::Exact(KnownEncoding::TextJavascript);
    pub const IMG_JPG: Encoding = Encoding::Exact(KnownEncoding::ImageJpeg);
    pub const IMG_PNG: Encoding = Encoding::Exact(KnownEncoding::ImagePng);
    pub const IMG_GIF: Encoding = Encoding::Exact(KnownEncoding::ImageGif);

    /// Sets the suffix of this encoding.
    pub fn with_suffix<IntoCowStr>(self, suffix: IntoCowStr) -> Self
    where
        IntoCowStr: Into<Cow<'static, str>>,
    {
        match self {
            Encoding::Exact(e) => Encoding::WithSuffix(e, suffix.into()),
            Encoding::WithSuffix(e, s) => {
                Encoding::WithSuffix(e, Cow::Owned(format!("{}{}", s, suffix.into())))
            }
        }
    }

    pub fn as_ref<'a, T>(&'a self) -> T
    where
        &'a Self: Into<T>,
    {
        self.into()
    }

    /// Returns `true`if the string representation of this encoding starts with
    /// the string representation of ther given encoding.
    pub fn starts_with(&self, with: &Encoding) -> bool {
        self.prefix() == with.prefix() && self.suffix().starts_with(with.suffix())
    }
    pub const fn prefix(&self) -> &KnownEncoding {
        match self {
            Encoding::Exact(e) | Encoding::WithSuffix(e, _) => e,
        }
    }
    pub fn suffix(&self) -> &str {
        match self {
            Encoding::Exact(_) => "",
            Encoding::WithSuffix(_, s) => s.as_ref(),
        }
    }
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Encoding::Exact(e) => f.write_str(e.as_ref()),
            Encoding::WithSuffix(e, s) => {
                f.write_str(e.as_ref())?;
                f.write_str(s)
            }
        }
    }
}

impl From<&'static str> for Encoding {
    fn from(s: &'static str) -> Self {
        for (i, v) in consts::MIMES.iter().enumerate().skip(1) {
            if let Some(suffix) = s.strip_prefix(v) {
                if suffix.is_empty() {
                    return Encoding::Exact(unsafe { std::mem::transmute(i as u8) });
                } else {
                    return Encoding::WithSuffix(
                        unsafe { std::mem::transmute(i as u8) },
                        suffix.into(),
                    );
                }
            }
        }
        if s.is_empty() {
            Encoding::Exact(KnownEncoding::Empty)
        } else {
            Encoding::WithSuffix(KnownEncoding::Empty, s.into())
        }
    }
}

impl From<String> for Encoding {
    fn from(mut s: String) -> Self {
        for (i, v) in consts::MIMES.iter().enumerate().skip(1) {
            if s.starts_with(v) {
                s.replace_range(..v.len(), "");
                if s.is_empty() {
                    return Encoding::Exact(unsafe { std::mem::transmute(i as u8) });
                } else {
                    return Encoding::WithSuffix(unsafe { std::mem::transmute(i as u8) }, s.into());
                }
            }
        }
        if s.is_empty() {
            Encoding::Exact(KnownEncoding::Empty)
        } else {
            Encoding::WithSuffix(KnownEncoding::Empty, s.into())
        }
    }
}

// impl From<ZInt> for Encoding {
//     fn from(i: ZInt) -> Self {
//         Encoding {
//             prefix: i,
//             suffix: "".into(),
//         }
//     }
// }
impl Default for Encoding {
    fn default() -> Self {
        Encoding::EMPTY
    }
}
