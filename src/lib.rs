//! Flatbuffers related tools

#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]
#![cfg_attr(rustfmt, rustfmt_skip)]

mod gen;
pub use gen::{RpcMethodDefines, RpcServiceImplDefines};

#[derive(Debug, Clone, PartialEq, Eq)]
///Possible parser errors
pub enum ParseError {
    ///Service definition is encountered, but there is no opening bracket
    NoStartingBracket,
    ///Cannot determine return type
    NoReturnType(String),
    ///Method definition has invalid arguments
    InvalidMethodArgs(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
///rpc method
pub struct RpcMethod {
    ///Method's name
    pub name: String,
    ///List of arguments
    pub arguments: Vec<String>,
    ///Return type
    pub return_type: String,
}

impl RpcMethod {
    fn parse(line: &str) -> Result<Self, ParseError> {
        let mut parts = line.split(':');
        let method_args = parts.next().unwrap();
        let return_type = match parts.next() {
            Some(return_type) => return_type.trim().trim_end_matches(';'),
            None => return Err(ParseError::NoReturnType(line.to_owned())),
        };
        let mut parts = method_args.split('(');
        let name = parts.next().unwrap().trim();
        let mut args = match parts.next() {
            Some(args) => args.trim(),
            None => return Err(ParseError::InvalidMethodArgs(method_args.to_owned())),
        };
        args = if let Some(args) = args.strip_suffix(')') {
            args
        } else {
            return Err(ParseError::InvalidMethodArgs(method_args.to_owned()))
        };
        let arguments = args.split(',').map(str::to_owned).collect();

        Ok(Self {
            name: name.to_owned(),
            arguments,
            return_type: return_type.to_owned()
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
///rpc_service definition
pub struct RpcService {
    ///Service name
    pub name: String,
    ///List of service methods
    pub methods: Vec<RpcMethod>
}

impl RpcService {
    ///Gets formatter to generate RPC method defines which are upper case constants corresponding
    ///to RPC method name.
    pub fn as_rpc_method_defines(&self) -> RpcMethodDefines<'_> {
        RpcMethodDefines {
            service: self
        }
    }
}

///rpc_service parser
pub struct ParserIter<T> {
    lines: T,
}

impl<I: AsRef<str>, T: Iterator<Item=I>> ParserIter<T> {
    ///Creates new parser from iterator over lines.
    pub fn new(lines: T) -> Self {
        Self {
            lines
        }
    }
}

impl<I: AsRef<str>, T: Iterator<Item=I>> Iterator for ParserIter<T> {
    type Item = Result<RpcService, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(line) = self.lines.next() {
            let line = line.as_ref().trim();
            if let Some(name) = line.strip_prefix("rpc_service") {
                if let Some(name_end_idx) = name.find('{') {
                    let name = name[..name_end_idx].trim();
                    let mut methods = Vec::new();

                    while let Some(method) = self.lines.next() {
                        let method = method.as_ref().trim();
                        if method == "}" {
                            break;
                        }

                        match RpcMethod::parse(method) {
                            Ok(method) => methods.push(method),
                            Err(error) => return Some(Err(error)),
                        }
                    }

                    return Some(Ok(RpcService {
                        name: name.to_owned(),
                        methods,
                    }));
                } else {
                    return Some(Err(ParseError::NoStartingBracket));
                }
            } else {
                continue
            }
        }

        None
    }
}
