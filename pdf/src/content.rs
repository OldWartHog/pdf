/// PDF content streams.
use std::fmt::{Display, Formatter};
use std::mem::replace;
use std::cmp::Ordering;
use std::io;
use itertools::Itertools;

use crate::error::*;
use crate::object::*;
use crate::parser::{Lexer, parse_with_lexer};
use crate::primitive::*;

/// Operation in a PDF content stream.
#[derive(Debug, Clone)]
pub struct Operation {
    pub operator: String,
    pub operands: Vec<Primitive>,
}

impl Operation {
    pub fn new(operator: String, operands: Vec<Primitive>) -> Operation {
        Operation{
            operator,
            operands,
        }
    }
}


/// Represents a PDF content stream - a `Vec` of `Operator`s
#[derive(Debug)]
pub struct Content {
    pub operations: Vec<Operation>,
}

impl Content {
    fn parse_from(data: &[u8], resolve: &impl Resolve) -> Result<Content> {
        let mut lexer = Lexer::new(data);

        let mut content = Content {operations: Vec::new()};
        let mut buffer = Vec::new();

        loop {
            let backup_pos = lexer.get_pos();
            let obj = parse_with_lexer(&mut lexer, resolve);
            match obj {
                Ok(obj) => {
                    // Operand
                    buffer.push(obj)
                }
                Err(e) => {
                    if e.is_eof() {
                        break;
                    }
                    // It's not an object/operand - treat it as an operator.
                    lexer.set_pos(backup_pos);
                    let operator = t!(lexer.next()).to_string();
                    let operation = Operation::new(operator, replace(&mut buffer, Vec::new()));
                    // Give operands to operation and empty buffer.
                    content.operations.push(operation.clone());
                }
            }
            match lexer.get_pos().cmp(&data.len()) {
                Ordering::Greater => err!(PdfError::ContentReadPastBoundary),
                Ordering::Less => (),
                Ordering::Equal => break
            }
        }
        Ok(content)
    }
}

impl Object for Content {
    /// Write object as a byte stream
    fn serialize<W: io::Write>(&self, _out: &mut W) -> Result<()> {unimplemented!()}
    /// Convert primitive to Self
    fn from_primitive(p: Primitive, resolve: &impl Resolve) -> Result<Self> {
        type ContentStream = Stream<()>;
        
        match p {
            Primitive::Array(parts) => {
                let mut content_data = Vec::new();
                for p in parts {
                    let part = t!(ContentStream::from_primitive(p, resolve));
                    let data = t!(part.data());
                    content_data.extend(data);
                }
                Content::parse_from(&content_data, resolve)
            }
            p => {
                Content::parse_from(
                    t!(t!(ContentStream::from_primitive(p, resolve)).data()),
                    resolve
                )
            }
        }
    }
}


impl Display for Content {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Content: ")?;
        for operation in &self.operations {
            write!(f, "{}", operation)?;
        }
        Ok(())
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{} : {}", self.operator, self.operands.iter().format(", "))
    }
}
