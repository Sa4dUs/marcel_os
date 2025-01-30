#[no_mangle]
pub extern "C" fn memset(ptr: *mut u8, value: u8, num: usize) -> *mut u8 {
    unsafe {
        let mut p = ptr;
        for _ in 0..num {
            *p = value;
            p = p.add(1);
        }
    }

    ptr
}

#[no_mangle]
pub extern "C" fn memcpy(dest: *mut u8, src: *const u8, num: usize) -> *mut u8 {
    unsafe {
        let mut dest_ptr = dest;
        let mut src_ptr = src;

        for _ in 0..num {
            *dest_ptr = *src_ptr;
            dest_ptr = dest_ptr.add(1);
            src_ptr = src_ptr.add(1);
        }
    }

    dest
}

#[no_mangle]
pub extern "C" fn memcmp(ptr1: *const u8, ptr2: *const u8, num: usize) -> i32 {
    unsafe {
        for i in 0..num {
            let byte1 = *ptr1.add(i);
            let byte2 = *ptr2.add(i);
            if byte1 != byte2 {
                return (byte1 as i32) - (byte2 as i32);
            }
        }
    }
    0
}
