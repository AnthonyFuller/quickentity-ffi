#![allow(clippy::not_unsafe_ptr_arg_deref)]

use quickentity_rs::{
    convert_2016_blueprint_to_modern, convert_2016_factory_to_modern, convert_to_qn,
    rpkg_structs::ResourceMeta,
    rt_structs::{RTBlueprint, RTFactory},
};
use serde::Serialize;
use serde_json::{from_str, from_value, ser::Formatter, Serializer, Value};
use std::io;

pub fn read_string_as_rtfactory(json: &std::ffi::CStr) -> RTFactory {
    let val = from_str::<Value>(json.to_str().expect("Input string is not valid UTF8"))
        .expect("Failed to read JSON string as JSON");

    if val.get("entityTemplates").is_some() {
        convert_2016_factory_to_modern(
            &from_value(val).expect("Failed to read JSON string as RT struct"),
        )
    } else {
        from_value(val).expect("Failed to read JSON string as RT struct")
    }
}

pub fn read_string_as_rtblueprint(json: &std::ffi::CStr) -> RTBlueprint {
    let val = from_str::<Value>(json.to_str().expect("Input string is not valid UTF8"))
        .expect("Failed to read JSON string as JSON");

    if val.get("entityTemplates").is_some() {
        convert_2016_blueprint_to_modern(
            &from_value(val).expect("Failed to read JSON string as RT struct"),
        )
    } else {
        from_value(val).expect("Failed to read JSON string as RT struct")
    }
}

pub fn read_string_as_meta(json: &std::ffi::CStr) -> ResourceMeta {
    from_str(json.to_str().expect("Input string is not valid UTF8"))
        .expect("Failed to read JSON string as JSON")
}

pub fn to_vec_float_format<W>(contents: &W) -> Vec<u8>
where
    W: ?Sized + Serialize,
{
    let mut writer = Vec::with_capacity(128);

    let mut ser = Serializer::with_formatter(&mut writer, FloatFormatter);
    contents.serialize(&mut ser).unwrap();

    writer
}

#[derive(Clone, Debug)]
struct FloatFormatter;

impl Formatter for FloatFormatter {
    #[inline]
    fn write_f32<W>(&mut self, writer: &mut W, value: f32) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(value.to_string().as_bytes())
    }

    #[inline]
    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(value.to_string().as_bytes())
    }

    /// Writes a number that has already been rendered to a string.
    #[inline]
    fn write_number_str<W>(&mut self, writer: &mut W, value: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        let x = value.parse::<f64>();
        if let Ok(y) = x {
            if value.parse::<u64>().is_err()
                || y.to_string() == value.parse::<u64>().unwrap().to_string()
            {
                writer
                    .write_all(
                        if y.to_string() == "-0" {
                            "0".to_string()
                        } else {
                            y.to_string()
                        }
                        .as_bytes(),
                    )
                    .unwrap();
            } else {
                writer.write_all(value.as_bytes()).unwrap();
            }
        } else {
            writer.write_all(value.as_bytes()).unwrap();
        }

        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn convert_entity_to_qn(
    factory_json: *const std::os::raw::c_char,
    factory_meta_json: *const std::os::raw::c_char,
    blueprint_json: *const std::os::raw::c_char,
    blueprint_meta_json: *const std::os::raw::c_char,
) -> *mut std::os::raw::c_char {
    let input_factory: &std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(factory_json) };
    let input_factory_meta: &std::ffi::CStr =
        unsafe { std::ffi::CStr::from_ptr(factory_meta_json) };
    let input_blueprint: &std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(blueprint_json) };
    let input_blueprint_meta: &std::ffi::CStr =
        unsafe { std::ffi::CStr::from_ptr(blueprint_meta_json) };

    let lossless = true;
    let factory = read_string_as_rtfactory(input_factory);
    let factory_meta = read_string_as_meta(input_factory_meta);
    let blueprint = read_string_as_rtblueprint(input_blueprint);
    let blueprint_meta = read_string_as_meta(input_blueprint_meta);

    let entity = convert_to_qn(
        &factory,
        &factory_meta,
        &blueprint,
        &blueprint_meta,
        lossless,
    )
    .expect("Error converting to QN entity");

    let entity_json = std::ffi::CString::new(to_vec_float_format(&entity))
        .expect("Error creating Entity JSON CString");
    entity_json.into_raw()
}

#[no_mangle]
pub extern "C" fn free_json_string(json_string: *mut std::os::raw::c_char) {
    unsafe {
        if json_string.is_null() {
            return;
        }
        drop(std::ffi::CString::from_raw(json_string))
    };
}
