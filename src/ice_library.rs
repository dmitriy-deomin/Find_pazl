use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use libloading::{Library, Symbol};

pub struct IceLibrary {
    ice: Library,
}

impl IceLibrary {
    pub fn new() -> Self {
        let ice = unsafe { Library::new("ice_secp256k1.dll") }.expect("Failed to load library");
        IceLibrary { ice }
    }

    pub(crate) fn init_secp256_lib(&self) {
        let init_secp256_lib: Symbol<unsafe extern "C" fn() -> ()> = unsafe { self.ice.get(b"init_secp256_lib") }.expect("Failed init");
        unsafe { init_secp256_lib() };
    }

    pub fn privatekey_to_address(&self, hex: &str) -> String {
        let privatekey_to_address: Symbol<unsafe extern "C" fn(i32, bool, *const i8) -> *mut i8> = unsafe { self.ice.get(b"privatekey_to_address") }.unwrap();
        let private_key = CString::new(hex).expect("Failed to create CString");
        let result = unsafe { privatekey_to_address(0, true, private_key.as_ptr()) };
        let result_str = unsafe { CStr::from_ptr(result) }.to_str().expect("Failed to convert C string to str");
        unsafe { libc::free(result as *mut libc::c_void) }; // Освобождаем память, выделенную внешней библиотекой
        result_str.to_owned() // Возвращаем владеющую строку
    }

    // pub fn privatekey_to_h160(&self, hexx: &str) -> [u8; 20] {
    //     let privatekey_to_h160: Symbol<unsafe extern "C" fn(i32, bool, *const c_char, *mut u8) -> ()> =
    //         unsafe { self.ice.get(b"privatekey_to_h160").unwrap() };
    //     let private_key = CString::new(hexx).expect("Не удалось создать CString");
    //     let mut res = [0u8; 20];
    //
    //     unsafe { privatekey_to_h160(0, true, private_key.as_ptr(), res.as_mut_ptr()) };
    //     res
    // }
    #[allow(invalid_value)]
    pub fn privatekey_to_h160(&self, hexx: &str) -> [u8; 20] {
        let privatekey_to_h160: Symbol<unsafe extern "C" fn(i32, bool, *const c_char, *mut u8) -> ()> =
            unsafe { self.ice.get(b"privatekey_to_h160").unwrap() };
        let mut res: [u8; 20] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        let private_key = CString::new(hexx).expect("Не удалось создать CString");

        unsafe { privatekey_to_h160(0, true, private_key.as_ptr(), res.as_mut_ptr()) };
        res
    }



}