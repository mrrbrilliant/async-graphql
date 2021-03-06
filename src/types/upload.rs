use crate::parser::types::UploadValue;
use crate::{registry, InputValueError, InputValueResult, InputValueType, Type, Value};
use std::borrow::Cow;
use std::io::Read;

/// Uploaded file
///
/// **Reference:** <https://github.com/jaydenseric/graphql-multipart-request-spec>
///
///
/// Graphql supports file uploads via `multipart/form-data`.
/// Enable this feature by accepting an argument of type `Upload` (single file) or
/// `Vec<Upload>` (multiple files) in your mutation like in the example blow.
///
///
/// # Example
/// *[Full Example](<https://github.com/async-graphql/examples/blob/master/models/files/src/lib.rs>)*
///
/// ```
/// use async_graphql::*;
///
/// struct MutationRoot;
///
/// #[Object]
/// impl MutationRoot {
///     async fn upload(&self, file: Upload) -> bool {
///         println!("upload: filename={}", file.filename());
///         true
///     }
/// }
///
/// ```
/// # Example Curl Request
/// Assuming you have defined your MutationRoot like in the example above,
/// you can now upload a file `myFile.txt` with the below curl command:
///
/// ```curl
/// curl 'localhost:8000' \
/// --form 'operations={
///         "query": "mutation ($file: Upload!) { upload(file: $file)  }",
///         "variables": { "file": null }}' \
/// --form 'map={ "0": ["variables.file"] }' \
/// --form '0=@myFile.txt'
/// ```
pub struct Upload(UploadValue);

impl Upload {
    /// Filename
    pub fn filename(&self) -> &str {
        self.0.filename.as_str()
    }

    /// Content type, such as `application/json`, `image/jpg` ...
    pub fn content_type(&self) -> Option<&str> {
        self.0.content_type.as_deref()
    }

    /// Returns the size of the file, in bytes.
    pub fn size(&self) -> std::io::Result<u64> {
        self.0.content.metadata().map(|meta| meta.len())
    }

    /// Convert to a `Read`.
    ///
    /// **Note**: this is a *synchronous/blocking* reader.
    pub fn into_read(self) -> impl Read + Sync + Send + 'static {
        self.0.content
    }

    #[cfg(feature = "unblock")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "unblock")))]
    /// Convert to a `AsyncRead`.
    pub fn into_async_read(self) -> impl futures::AsyncRead + Sync + Send + 'static {
        blocking::Unblock::new(self.0.content)
    }
}

impl Type for Upload {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("Upload")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|_| registry::MetaType::Scalar {
            name: Self::type_name().to_string(),
            description: None,
            is_valid: |value| matches!(value, Value::Upload(_)),
        })
    }
}

impl InputValueType for Upload {
    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        let value = value.unwrap_or_default();
        if let Value::Upload(upload) = value {
            Ok(Upload(upload))
        } else {
            Err(InputValueError::ExpectedType(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::Null
    }
}
