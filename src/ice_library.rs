use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
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

    #[allow(invalid_value)]
    pub fn privatekey_to_h160(&self, hexx: &str) -> [u8; 20] {
        let privatekey_to_h160: Symbol<unsafe extern "C" fn(i32, bool, *const c_char, *mut u8) -> ()> =
            unsafe { self.ice.get(b"privatekey_to_h160").unwrap() };
        let mut res: [u8; 20] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        let private_key = CString::new(hexx).expect("Не удалось создать CString");

        unsafe { privatekey_to_h160(0, true, private_key.as_ptr(), res.as_mut_ptr()) };
        res
    }
    #[allow(invalid_value)]
    pub fn pubkey_to_h160(&self, hexx: [u8; 65]) -> [u8; 20] {
        let pubkey_to_h160: Symbol<unsafe extern "C" fn(i32, bool, *const c_char, *mut u8) -> ()> =
            unsafe { self.ice.get(b"pubkey_to_h160").unwrap() };
        let mut res: [u8; 20] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        let pubkey = CString::new(hexx).expect("Не удалось создать CString");

        unsafe { pubkey_to_h160(0, true, pubkey.as_ptr(), res.as_mut_ptr()) };
        res
    }
    pub fn publickey_to_address(&self, addr_type: c_int, is_compressed: bool, pubkey: &[u8]) -> String {
        let pubkey_to_address: Symbol<unsafe extern "C" fn(c_int, bool, *const u8) -> *mut c_char> =
            unsafe { self.ice.get(b"pubkey_to_address") }.unwrap();

        unsafe {
            let result_ptr = pubkey_to_address(addr_type, is_compressed, pubkey.as_ptr());
            let address = CString::from_raw(result_ptr).into_string().expect("Failed to convert C string to Rust string");
            address
        }
    }
    #[allow(invalid_value)]
    pub fn point_sequential_increment(&self, pubk: [u8; 65]) -> [u8; 65*3] {
        let point_sequential_increment: Symbol<unsafe extern "C" fn(i32,*const c_char, *mut u8) -> ()> =
            unsafe { self.ice.get(b"pubkey_to_h160").unwrap() };
        let mut res: [u8; 65*3] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        let pubkey = CString::new(pubk).expect("Не удалось создать CString");

        unsafe { point_sequential_increment(3,pubkey.as_ptr(), res.as_mut_ptr()) };
        res
    }
    #[allow(invalid_value)]
    pub fn scalar_multiplication(&self, hexx: &str) -> [u8; 65] {
        let scalar_multiplication: Symbol<unsafe extern "C" fn(*const c_char,*mut u8) -> ()> =
            unsafe { self.ice.get(b"scalar_multiplication").unwrap() };
        let mut res: [u8; 65] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        let pvk = CString::new(hexx).expect("Failed to create CString");

        unsafe { scalar_multiplication(pvk.as_ptr(), res.as_mut_ptr()) };
        res
    }



}