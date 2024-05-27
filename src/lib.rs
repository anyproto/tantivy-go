use std::ffi::{c_char, CStr, CString};
use tantivy::columnar::ColumnType::Str;
use log::debug;

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

    fn get_arr(&self) -> Vec<String> {
        return vec!["sd1".to_owned(), "sd2".to_owned()]
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
pub extern "C" fn example_get_arr(example_ptr: *const Example) -> *const *const c_char {
    let example = unsafe {
        assert!(!example_ptr.is_null());
        &*example_ptr
    };

    debug!("{:?}", example.get_arr());

    // Предположим, что example.get_arr() возвращает Vec<String>
    let c_strings: Vec<CString> = example.get_arr().iter()
        .map(|s| CString::new(s.as_str()).expect("CString::new failed"))
        .collect();

    // Создаем вектор указателей на строки C
    let mut c_string_ptrs: Vec<*const c_char> = c_strings.iter()
        .map(|s| s.as_ptr())
        .collect();

    // Добавляем завершающий NULL указатель
    c_string_ptrs.push(std::ptr::null());

    // Помещаем вектор в кучу, чтобы он не был освобожден при завершении функции
    let c_string_ptrs_boxed = c_string_ptrs.into_boxed_slice();

    // Получаем указатель на первый элемент массива указателей на строки C
    let c_string_ptrs_ptr = c_string_ptrs_boxed.as_ptr();

    // Запрещаем освобождение вектора при выходе из функции
    std::mem::forget(c_strings); // Не освобождаем CString
    std::mem::forget(c_string_ptrs_boxed);

    c_string_ptrs_ptr
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

/// # Safety
///
#[no_mangle]
pub unsafe extern "C" fn init() -> u8 {
    let mut log_level: &str = "info";
    let parse_val: String;
    if let Ok(existing_value) = std::env::var("ELV_RUST_LOG") {
        parse_val = existing_value;
        log_level = &parse_val;
    }
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .try_init();
    0
}
