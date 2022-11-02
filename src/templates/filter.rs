use std::collections::HashMap;

use camino::Utf8Path;
use tera::{from_value, to_value, Result, Tera, Value};

fn file_name(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let value = from_value::<String>(value.clone())?;
    let file_name = Utf8Path::new(&value).file_name().map(ToOwned::to_owned);
    to_value(file_name).map_err(Into::into)
}

pub fn register_filters(tera: &mut Tera) {
    tera.register_filter("file_name", file_name);
}
