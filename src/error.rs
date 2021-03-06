//! Standard error type for ocl.
//!

use std;
use std::convert::Into;
use std::default::Default;
use num::FromPrimitive;
use ::Status;


/// `ocl::Error` result type.
pub type Result<T> = std::result::Result<T, self::Error>;


/// An enum containing either a `String` or one of several other error types.
///
/// Implements the usual error traits.
///
/// ## Stability
///
/// The `String` variant may eventually be removed. Many more variants and
/// sub-types will be added as time goes on and things stabilize.
///
/// `Status` will eventually be changed internally to contain a sub-error type
/// unique to each function which generates it (yeah that'll be fun to
/// implement).
///
/// `UnspecifiedDimensions` may be moved into a sub-type.
///
/// For now, don't assume the existence of or check for any of the above.
///
#[derive(Debug)]
pub enum Error {
    Conversion(String),
    Status {
        status: Status, status_string: String, fn_name: &'static str, fn_info: String, desc: String
    },
    String(String),
    Nul(std::ffi::NulError),
    Io(std::io::Error),
    FromUtf8Error(std::string::FromUtf8Error),
    UnspecifiedDimensions,
    IntoStringError(std::ffi::IntoStringError),
}

impl self::Error {
    /// Returns a new `Error` with the description string: `desc`.
    pub fn new<S: Into<String>>(desc: S) -> self::Error {
        self::Error::String(desc.into())
    }

    /// Returns a new `ocl::Result::Err` containing an `ocl::Error::String`
    /// variant with the given description.
    pub fn err<T, S: Into<String>>(desc: S) -> self::Result<T> {
        Err(Error::String(desc.into()))
    }

    /// Returns a new `ocl::Result::Err` containing an `ocl::Error` with the
    /// given error code and description.
    pub fn err_status<T: Default, S: Into<String>>(errcode: i32, fn_name: &'static str, fn_info: S)
            -> self::Result<T>
    {
        let status = match Status::from_i32(errcode) {
            Some(s) => s,
            None => panic!("ocl::core::errcode_try(): Invalid error code: '{}'. Aborting.", errcode),
        };

        if let Status::CL_SUCCESS = status {
            Ok(T::default())
        } else {
            let fn_info = fn_info.into();
            let desc = fmt_status_desc(status.clone(), fn_name, &fn_info);
            let status_string = format!("{:?}", status);

            Err(Error::Status {
                    status: status,
                    status_string: status_string,
                    fn_name: fn_name,
                    fn_info: fn_info,
                    desc: desc
            })
        }
    }

    /// Returns a new `ocl::Result::Err` containing an
    /// `ocl::Error::Conversion` variant with the given description.
    pub fn err_conversion<T, S: Into<String>>(desc: S) -> self::Result<T> {
        Err(Error::Conversion(desc.into()))
    }

    /// If this is a `String` variant, concatenate `txt` to the front of the
    /// contained string. Otherwise, do nothing at all.
    pub fn prepend<'s, S: AsRef<&'s str>>(&'s mut self, txt: S) {
        if let &mut Error::String(ref mut string) = self {
            string.reserve_exact(txt.as_ref().len());
            let old_string_copy = string.clone();
            string.clear();
            string.push_str(txt.as_ref());
            string.push_str(&old_string_copy);
        }
    }

    // /// Returns the error status const code name or nothing.
    // pub fn status_code(&self) -> String {
    //     match *self {
    //         Error::Status { ref status, .. } => format!("{:?}", status),
    //         _ => format!("{:?}", self),
    //     }
    // }

    /// Returns the error status code for `Status` variants.
    pub fn status(&self) -> Option<Status> {
        match *self {
            Error::Status { ref status, .. } => Some(status.clone()),
            _ => None,
        }
    }
}

impl std::error::Error for self::Error {
    fn description(&self) -> &str {
        match *self {
            Error::Conversion(ref desc) => desc,
            Error::Nul(ref err) => err.description(),
            Error::Io(ref err) => err.description(),
            Error::FromUtf8Error(ref err) => err.description(),
            Error::IntoStringError(ref err) => err.description(),
            Error::Status { ref status_string, .. } => status_string,
            Error::String(ref desc) => desc,
            Error::UnspecifiedDimensions => "Cannot convert to a valid set of dimensions. \
                Please specify some dimensions.",
            // _ => panic!("OclError::description()"),
        }
    }
}

impl Into<String> for self::Error {
    fn into(self) -> String {
        use std::error::Error;
        self.description().to_string()
    }
}

impl From<String> for self::Error {
    fn from(desc: String) -> self::Error {
        self::Error::new(desc)
    }
}

impl<'a> From<&'a str> for self::Error {
    fn from(desc: &'a str) -> self::Error {
        self::Error::new(String::from(desc))
    }
}

impl From<std::ffi::NulError> for self::Error {
    fn from(err: std::ffi::NulError) -> self::Error {
        self::Error::Nul(err)
    }
}

impl From<std::io::Error> for self::Error {
    fn from(err: std::io::Error) -> self::Error {
        self::Error::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for self::Error {
    fn from(err: std::string::FromUtf8Error) -> self::Error {
        self::Error::FromUtf8Error(err)
    }
}

impl From<std::ffi::IntoStringError> for self::Error {
    fn from(err: std::ffi::IntoStringError) -> self::Error {
        self::Error::IntoStringError(err)
    }
}

impl std::fmt::Display for self::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::error::Error;
        f.write_str(self.description())
    }
}

// impl std::fmt::Debug for self::Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         use std::error::Error;
//         f.write_str(self.description())
//     }
// }


static SDK_DOCS_URL_PRE: &'static str = "https://www.khronos.org/registry/cl/sdk/1.2/docs/man/xhtml/";
static SDK_DOCS_URL_SUF: &'static str = ".html#errors";

fn fmt_status_desc(status: Status, fn_name: &'static str, fn_info: &str) -> String {
    let fn_info_string = if fn_info.is_empty() == false {
        format!("(\"{}\")", fn_info)
    } else {
        String::with_capacity(0)
    };

    format!("\n\n\
        ################################ OPENCL ERROR ############################### \
        \n\nError executing function: {}{}  \
        \n\nStatus error code: {:?} ({})  \
        \n\nPlease visit the following url for more information: \n\n{}{}{}  \n\n\
        ############################################################################# \n",
        fn_name, fn_info_string, status.clone(), status as i32,
        SDK_DOCS_URL_PRE, fn_name, SDK_DOCS_URL_SUF)
}
