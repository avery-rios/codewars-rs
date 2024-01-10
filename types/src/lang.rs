use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

macro_rules! known_lang {
    ($($n:ident => $id:literal),+) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[non_exhaustive]
        pub enum KnownLangId {
            $(#[serde(rename = $id)] $n,)+
        }
        impl KnownLangId {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$n => $id,)+
                }
            }
            pub fn from_lang_id(lang: &str) -> Option<Self> {
                match lang {
                    $($id => Some(Self::$n),)+
                    _ => None
                }
            }
        }
    };
}

known_lang! {
    Agda => "agda",
    BrainFuck => "bf",
    C => "c",
    Cfml => "cfml",
    Clojure => "clojure",
    Cobol => "cobol",
    CoffeeScript => "coffeescript",
    CommonLisp => "commonlisp",
    Coq => "coq",
    Cpp => "cpp",
    Crystal => "crystal",
    CSharp => "csharp",
    D => "d",
    Dart => "dart",
    Elixir => "elixir",
    Elm => "elm",
    Erlang => "erlang",
    Factor => "factor",
    Forth => "forth",
    Fortran => "fortran",
    FSharp => "fsharp",
    Go => "go",
    Groovy => "groovy",
    Haskell => "haskell",
    Haxe => "haxe",
    Idris => "idris",
    Java => "java",
    JavaScript => "javascript",
    Julia => "julia",
    Kotlin => "kotlin",
    LambdaCalc => "lambdacalc",
    Lean => "lean",
    Lua => "lua",
    Nasm => "nasm",
    Nim => "nim",
    ObjC => "objc",
    OCaml => "ocaml",
    Pascal => "pascal",
    Perl => "perl",
    Php => "php",
    PowerShell => "powershell",
    Prolog => "prolog",
    PureScript => "purescript",
    Python => "python",
    R => "r",
    Racket => "racket",
    Raku => "raku",
    Reason => "reason",
    RiscV => "riscv",
    Ruby => "ruby",
    Rust => "rust",
    Scala => "scala",
    Shell => "shell",
    Solidity => "solidity",
    Sql => "sql",
    Swift => "swift",
    TypeScript => "typescript",
    Vb => "vb"
}

#[derive(Debug)]
pub struct UnknownLangErr;
impl Display for UnknownLangErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Unknown language id")
    }
}
impl std::error::Error for UnknownLangErr {}

impl Display for KnownLangId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
impl FromStr for KnownLangId {
    type Err = UnknownLangErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_lang_id(s).ok_or(UnknownLangErr)
    }
}
impl<'a> TryFrom<&'a str> for KnownLangId {
    type Error = UnknownLangErr;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::from_lang_id(value).ok_or(UnknownLangErr)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LangId {
    Known(KnownLangId),
    Unknown(String),
}
impl LangId {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Known(l) => l.as_str(),
            Self::Unknown(s) => s.as_str(),
        }
    }
}
impl Serialize for LangId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}
impl<'de> Deserialize<'de> for LangId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = LangId;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("expect language id")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(match KnownLangId::from_lang_id(v) {
                    Some(l) => LangId::Known(l),
                    None => LangId::Unknown(v.to_owned()),
                })
            }
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(match KnownLangId::from_lang_id(&v) {
                    Some(l) => LangId::Known(l),
                    None => LangId::Unknown(v),
                })
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}
