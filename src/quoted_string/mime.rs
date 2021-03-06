use lut::{Table, Any};
use lookup_tables::{
    MediaTypeChars,
    QText,
    QTextWs,
    DQuoteOrEscape, Ws,
    Token
};
use qs::error::CoreError;
use qs::spec::{
    PartialCodePoint,
    ParsingImpl,
    State,
    WithoutQuotingValidator,
    QuotingClassifier, QuotingClass,
};

use super::{MimeParsingExt, FWSState};

/// a type providing a `WithoutQuotingValidator` for token wrt. the mime grammar
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct MimeTokenValidator;

impl MimeTokenValidator {
    /// create a new MimeTokenValidator
    pub fn new() -> Self {
        Default::default()
    }
}

impl WithoutQuotingValidator for MimeTokenValidator {
    fn next(&mut self, pcp: PartialCodePoint) -> bool {
        MediaTypeChars::check_at(pcp.as_u8() as usize, Token)
    }
    fn end(&self) -> bool {
        true
    }
}


/// a type providing a `QuotingClassifier` impl wrt. the obs mime grammar
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct MimeObsQuoting;

impl QuotingClassifier for MimeObsQuoting {
    fn classify_for_quoting(pcp: PartialCodePoint) -> QuotingClass {
        let iu8 = pcp.as_u8();
        if MediaTypeChars::check_at(iu8 as usize, QTextWs) {
            QuotingClass::QText
        } else if iu8 <= 0x7f {
            QuotingClass::NeedsQuoting
        } else {
            QuotingClass::Invalid
        }
    }
}

/// a type providing a `QuotingClassifier` impl wrt. the internationalized, obs mime grammar
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct MimeObsUtf8Quoting;

impl QuotingClassifier for MimeObsUtf8Quoting {
    fn classify_for_quoting(pcp: PartialCodePoint) -> QuotingClass {
        let iu8 = pcp.as_u8();
        if iu8 > 0x7f || MediaTypeChars::check_at(iu8 as usize, QTextWs) {
            QuotingClass::QText
        } else {
            QuotingClass::NeedsQuoting
        }
    }
}



macro_rules! def_mime_parsing {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            utf8 = $utf8:tt;
            obsolte_syntax = $obs:tt;
        }
        fn can_be_quoted($nm:ident: PartialCodePoint) -> bool
            $body:block
    ) => (
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
        pub struct $name(FWSState);
        impl MimeParsingExt for $name {
            const ALLOW_UTF8: bool = $utf8;
            const OBS: bool = $obs;

            fn custom_state(state: FWSState, emit: bool) -> (State<Self>, bool) {
                (State::Custom($name(state)), emit)
            }
        }

        impl ParsingImpl for $name {
            fn can_be_quoted($nm: PartialCodePoint) -> bool {
                $body
            }

            fn handle_normal_state(bch: PartialCodePoint) -> Result<(State<Self>, bool), CoreError> {
                <Self as MimeParsingExt>::handle_normal_state(bch)
            }

            fn advance(&self, bch: PartialCodePoint) -> Result<(State<Self>, bool), CoreError> {
                self.0.advance(bch)
            }
        }
    );
}

def_mime_parsing! {
    /// a type providing a `ParsingImpl`/`MimeParsingExt` impl wrt. the obs mime grammar
    pub struct MimeObsParsing {
        utf8 = false;
        obsolte_syntax = true;
    }
    fn can_be_quoted(bch: PartialCodePoint) -> bool {
        // obs syntax allows any us-ascii in quoted-pairs
        bch.as_u8() <= 0x7f
    }
}

def_mime_parsing! {
    /// a type providing a `ParsingImpl`/`MimeParsingExt` impl wrt. the internationalized obs mime grammar
    pub struct MimeObsParsingUtf8 {
        utf8 = true;
        obsolte_syntax = true;
    }
    fn can_be_quoted(bch: PartialCodePoint) -> bool {
        // Internationalized Mail does not extend quoted-pairs just qtext ...
        // obs syntax allows any us-ascii in quoted-pairs
        bch.as_u8() <= 0x7f
    }
}

def_mime_parsing! {
    /// a type providing a `ParsingImpl`/`MimeParsingExt` impl wrt. the modern, us-ascii mime grammar
    pub struct MimeParsing {
        utf8 = false;
        obsolte_syntax = false;
    }
    fn can_be_quoted(bch: PartialCodePoint) -> bool {
        // VCHAR / WS == QText + Ws + DQuoteOrEscape
        let idx = bch.as_u8() as usize;
        MediaTypeChars::check_at(idx, Any::new(Ws) | QText | DQuoteOrEscape)
    }
}

def_mime_parsing! {
    /// a type providing a `ParsingImpl`/`MimeParsingExt` impl wrt. the internationalized, modern mime grammar
    pub struct MimeParsingUtf8 {
        utf8 = true;
        obsolte_syntax = false;
    }
    fn can_be_quoted(bch: PartialCodePoint) -> bool {
        // Internationalized Mail does not extend quoted-pairs just qtext ...
        let idx = bch.as_u8() as usize;
        MediaTypeChars::check_at(idx, Any::new(Ws) | QText | DQuoteOrEscape)
    }
}

