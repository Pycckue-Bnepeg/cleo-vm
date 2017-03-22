// use std::error;
use std::fmt;

#[derive(Debug)]
pub enum OpcodeHandlerErr {
	CannotParseArg,
	UndefinedCondArg,
	NotCorrectType(String),
}

impl fmt::Display for OpcodeHandlerErr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			OpcodeHandlerErr::CannotParseArg => write!(f, "Cannot parse arguments of opcode"),
			OpcodeHandlerErr::UndefinedCondArg => write!(f, "Undefined an argument of condition"),
			OpcodeHandlerErr::NotCorrectType(ref text) => write!(f, "This type is not correct. Expected type is {}", text),
		}
	}
}

/* TODO
impl error::Error for OpcodeHandlerErr {

}
*/