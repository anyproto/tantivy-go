use std::ffi::{c_char, CStr, CString};
use tantivy::columnar::ColumnType::Str;

mod build;

trait ExampleModifier {
    fn change_id(&mut self, newId: &str);
}
struct Example {
    pub(crate) id: String
}

impl Example {
    fn new(id: &str) -> Example {
        return Example {
            id: String::from(id)
        }
    }
}

impl ExampleModifier for Example {
    fn change_id(&mut self, new_id: &str) {
        self.id=String::from(new_id)
    }
}

#[no_mangle]
pub extern "C" fn create_example(name: *const c_char) -> *mut Example {
    let c_str = unsafe { CStr::from_ptr(name) };
    let name = c_str.to_str().expect("Failed to convert CStr to str");
    Box::into_raw(Box::new(Example::new(name)))
}

#[no_mangle]
pub extern "C" fn example_set_name(example_ptr: *mut Example, name_ptr: *const c_char) {
    let example = unsafe {
        assert!(!example_ptr.is_null());
        &mut *example_ptr
    };

    let name = unsafe {
        assert!(!name_ptr.is_null());
        CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
    };

    example.change_id(&name);
}

#[no_mangle]
pub extern "C" fn example_get_name(example_ptr: *const Example) -> *const c_char {
    let example = unsafe {
        assert!(!example_ptr.is_null());
        &*example_ptr
    };

    let name = example.id.as_str(); // Получаем имя

    // Конвертируем имя в строку C
    let c_name = CString::new(name).expect("CString::new failed");
    c_name.into_raw()
}

#[no_mangle]
pub extern "C" fn delete_example(ptr: *mut Example) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr);
    }
}
