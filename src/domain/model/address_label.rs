use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct AddressLabel {
    address: String,
    label_type: String,
    label_name: String,
}

impl AddressLabel {
    pub(crate) fn address(&self) -> String {
        self.address.replace('\\', "0")
    }

    pub(crate) fn label_type(&self) -> &str {
        &self.label_type
    }

    pub(crate) fn label_name(&self) -> &str {
        &self.label_name
    }
}
